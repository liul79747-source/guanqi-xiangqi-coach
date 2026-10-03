mod engine;
mod position;
mod vision;
use std::{path::PathBuf,sync::{Arc,Mutex,atomic::{AtomicU64,Ordering}},thread,time::Duration};
use tauri::{Manager,Emitter};
use serde::Serialize;
struct State{libs:PathBuf,analysis:Mutex<()>,watch_id:AtomicU64}
pub fn diagnose()->Result<String,String>{
    let exe=std::env::current_exe().map_err(|e|e.to_string())?;
    let libs=exe.parent().ok_or("无法读取程序目录")?.join("libs");
    vision::init(&libs)?;
    let settings=engine::Settings{depth:8,time:500,threads:1,hash:32,cloud:false,cloud_timeout:1};
    let result=engine::analyze(&libs,"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",&settings)?;
    if result.bestmove.is_empty(){return Err("引擎没有返回走法".into());}
    serde_json::to_string(&serde_json::json!({"ok":true,"model":"loaded","engine":result.source,"bestmove":result.bestmove,"depth":result.depth,"resources":libs})).map_err(|e|e.to_string())
}
#[derive(Serialize)] #[serde(rename_all="camelCase")] struct Resources{engine:bool,model:bool,runtime:bool,desktop:bool,resource_dir:String}
#[derive(Serialize)] struct WindowInfo{id:u32,title:String,app:String}
#[derive(Serialize,Clone)] #[serde(rename_all="camelCase")] struct Capture{
    fen:String,
    confidence:f32,
    #[serde(skip_serializing_if="Option::is_none")] recovered_plies:Option<u8>,
    #[serde(skip_serializing_if="Option::is_none")] warning:Option<String>,
}
#[tauri::command] fn resources(state:tauri::State<'_,Arc<State>>)->Resources{
    let p=&state.libs;Resources{engine:p.join("pikafish/pikafish-windows.exe").is_file()&&p.join("pikafish/pikafish.nnue").is_file(),model:p.join("models/board.onnx").is_file(),runtime:p.join("runtime/onnxruntime.dll").is_file(),desktop:true,resource_dir:p.display().to_string()}
}
#[tauri::command] async fn analyze(state:tauri::State<'_,Arc<State>>,fen:String,settings:engine::Settings)->Result<engine::Analysis,String>{
    let state=Arc::clone(&state);
    tauri::async_runtime::spawn_blocking(move||{let _guard=state.analysis.try_lock().map_err(|_|"上一局面仍在计算，请稍后再试")?;engine::analyze(&state.libs,&fen,&settings)}).await.map_err(|e|e.to_string())?
}
#[tauri::command] async fn list_windows()->Result<Vec<WindowInfo>,String>{
    tauri::async_runtime::spawn_blocking(||{let wins=xcap::Window::all().map_err(|e|e.to_string())?;Ok(wins.iter().filter_map(|w|{let title=w.title().ok()?;if title.is_empty()||title.contains("观棋 ·")||w.is_minimized().unwrap_or(true){return None;}Some(WindowInfo{id:w.id().ok()?,title,app:w.app_name().unwrap_or_default()})}).collect())}).await.map_err(|e|e.to_string())?
}
fn capture_board(libs:&std::path::Path,id:u32)->Result<(position::Board,f32),String>{
    vision::init(libs)?;
    let window=xcap::Window::all().map_err(|e|e.to_string())?.into_iter().find(|w|w.id().ok()==Some(id)).ok_or("目标窗口已关闭，请重新选择")?;
    if window.is_minimized().unwrap_or(true){return Err("目标窗口已最小化，请恢复后重试".into());}
    let image=window.capture_image().map_err(|e|format!("截图失败：{e}"))?;vision::recognize(image)
}
fn side_char(side:&str)->Result<char,String>{match side{"w"=>Ok('w'),"b"=>Ok('b'),_=>Err("行棋方必须是 w 或 b".into())}}
#[tauri::command] async fn capture(state:tauri::State<'_,Arc<State>>,window_id:u32,side:String)->Result<Capture,String>{
    let side=side_char(&side)?;let libs=state.libs.clone();
    tauri::async_runtime::spawn_blocking(move||{let (board,confidence)=capture_board(&libs,window_id)?;Ok(Capture{fen:position::fen(&board,side),confidence,recovered_plies:None,warning:None})}).await.map_err(|e|e.to_string())?
}
#[tauri::command] fn stop_watch(state:tauri::State<'_,Arc<State>>){state.watch_id.fetch_add(1,Ordering::SeqCst);}
#[tauri::command] async fn start_watch(app:tauri::AppHandle,state:tauri::State<'_,Arc<State>>,window_id:u32,side:String)->Result<(),String>{
    let initial_side=side_char(&side)?;let shared=Arc::clone(&state);let libs=shared.libs.clone();
    tauri::async_runtime::spawn_blocking(move||vision::init(&libs)).await.map_err(|e|e.to_string())??;
    let id=shared.watch_id.fetch_add(1,Ordering::SeqCst)+1;
    thread::spawn(move||{
        let mut last:Option<position::Board>=None;let mut candidate:Option<position::Board>=None;let mut samples:std::collections::VecDeque<position::Board>=std::collections::VecDeque::with_capacity(3);let mut side=initial_side;let mut last_error=String::new();let mut capture_failed=false;let mut pending_warning:Option<String>=None;
        while shared.watch_id.load(Ordering::SeqCst)==id {
            match capture_board(&shared.libs,window_id){
                Ok((board,confidence))=>{
                    if shared.watch_id.load(Ordering::SeqCst)!=id{break;}
                    if capture_failed{let _=app.emit("capture-error",pending_warning.as_deref().unwrap_or(""));capture_failed=false;last_error=pending_warning.clone().unwrap_or_default();}
                    samples.push_back(board);
                    while samples.len()>3{samples.pop_front();}
                    if samples.len()<3{candidate=None;thread::sleep(Duration::from_millis(650));continue;}
                    let Some((stable,stability))=position::consensus(&samples) else {candidate=None;thread::sleep(Duration::from_millis(650));continue;};
                    let confirmed=candidate.as_ref()==Some(&stable);
                    candidate=Some(stable.clone());
                    if confirmed && last.as_ref()!=Some(&stable){
                        let mut recovered_plies=None;
                        let mut warning=None;
                        if let Some(previous)=last.as_ref(){
                            if let Some((next,plies))=position::infer_plies(previous,&stable,side){
                                side=next;
                                if plies>1{
                                    recovered_plies=Some(plies);
                                    warning=Some(format!("已从稳定棋盘中补偿识别到漏过的 {plies} 手，请核对棋谱记录。"));
                                }
                            }else{
                                // Never keep a stale baseline: even ambiguous or skipped moves
                                // must rebase so the next stable position can be tracked.
                                side=if side=='w'{'b'}else{'w'};
                                warning=Some("棋局已自动重新同步并继续识别；漏帧超出自动恢复范围，行棋方已按一步推断，请核对。".into());
                            }
                        }
                        let _=app.emit("board-update",Capture{fen:position::fen(&stable,side),confidence:confidence*stability,recovered_plies,warning:warning.clone()});
                        last=Some(stable);pending_warning=warning.clone();last_error=warning.unwrap_or_default();
                        thread::sleep(Duration::from_millis(650));
                        continue;
                    }
                },
                Err(error)=>{candidate=None;samples.clear();capture_failed=true;if error!=last_error&&shared.watch_id.load(Ordering::SeqCst)==id{let _=app.emit("capture-error",&error);last_error=error;}}
            }
            thread::sleep(Duration::from_millis(650));
        }
    });Ok(())
}
pub fn run(){
    tauri::Builder::default().setup(|app|{
        let bundled=app.path().resource_dir()?.join("libs");
        let portable=std::env::current_exe()?.parent().ok_or("无法读取程序目录")?.join("libs");
        let dev=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../libs");
        let complete=|p:&std::path::Path|p.join("models/board.onnx").is_file()&&p.join("runtime/onnxruntime.dll").is_file();
        let libs=if cfg!(debug_assertions)&&dev.exists(){dev}else if complete(&bundled){bundled}else if complete(&portable){portable}else{bundled};
        app.manage(Arc::new(State{libs,analysis:Mutex::new(()),watch_id:AtomicU64::new(0)}));Ok(())
    }).invoke_handler(tauri::generate_handler![resources,analyze,list_windows,capture,start_watch,stop_watch])
    .on_window_event(|window,event|{if matches!(event,tauri::WindowEvent::Destroyed){window.state::<Arc<State>>().watch_id.fetch_add(1,Ordering::SeqCst);}})
    .run(tauri::generate_context!()).expect("无法启动观棋桌面程序");
}
