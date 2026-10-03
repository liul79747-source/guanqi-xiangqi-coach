use std::{io::{BufRead, BufReader, Write}, path::Path, process::{Child, Command, Stdio}, sync::mpsc, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};
use crate::position;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Settings { pub depth:u32, pub time:u64, pub threads:u32, pub hash:u32, pub cloud:bool, pub cloud_timeout:u64 }
impl Settings {
    pub fn validate(&self)->Result<(),String> { if !(1..=60).contains(&self.depth)||!(100..=30000).contains(&self.time)||!(1..=16).contains(&self.threads)||!(16..=1024).contains(&self.hash)||!(1..=10).contains(&self.cloud_timeout) {Err("分析参数超出有效范围".into())} else {Ok(())} }
}
#[derive(Clone, Debug, Serialize, Default)]
pub struct Analysis { pub fen:String, pub bestmove:String, pub pv:Vec<String>, pub score:i32, pub mate:Option<i32>, pub depth:u32, pub time:u64, pub source:String, #[serde(skip_serializing_if="Option::is_none")] pub warning:Option<String> }
pub fn cloud(fen:&str, timeout:u64)->Result<Analysis,String> {
    let start=Instant::now();
    let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(timeout)).build().map_err(|e|e.to_string())?;
    let text=client.get("https://www.chessdb.cn/chessdb.php").query(&[("action","querypv"),("board",fen)]).send().and_then(|r|r.error_for_status()).and_then(|r|r.text()).map_err(|e|e.to_string())?;
    let mut result=Analysis{fen:fen.into(),source:"ChessDB 云库".into(),..Default::default()};
    for pair in text.trim_matches(['\0','\n','\r',' ']).split(',') {
        if let Some((key,value))=pair.split_once(':') {match key {"score"=>result.score=value.parse().map_err(|_|"云库评分无效")?,"depth"=>result.depth=value.parse().unwrap_or(0),"pv"=>result.pv=value.split('|').filter(|m|position::valid_move(m)).map(String::from).collect(),_=>{}}}
    }
    result.bestmove=result.pv.first().cloned().ok_or("云库未收录或暂时不可用")?;
    result.time=start.elapsed().as_millis() as u64;Ok(result)
}
struct Process(Child);
impl Drop for Process {fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
pub fn local(libs:&Path, fen:&str, config:&Settings)->Result<Analysis,String> {
    let engine=libs.join("pikafish/pikafish-windows.exe");let nnue=libs.join("pikafish/pikafish.nnue");
    if !engine.is_file()||!nnue.is_file(){return Err("Pikafish 引擎或 NNUE 文件缺失，请检查 libs/pikafish".into());}
    let mut cmd=Command::new(&engine);cmd.current_dir(engine.parent().unwrap_or(libs)).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)] {use std::os::windows::process::CommandExt;cmd.creation_flags(0x08000000);}
    let mut process=Process(cmd.spawn().map_err(|e|format!("无法启动 Pikafish：{e}"))?);
    let mut stdin=process.0.stdin.take().ok_or("无法打开引擎输入")?;let stdout=process.0.stdout.take().ok_or("无法打开引擎输出")?;
    let (tx,rx)=mpsc::channel();std::thread::spawn(move||{for line in BufReader::new(stdout).lines(){match line {Ok(line)=>{if tx.send(line).is_err(){break}},Err(_)=>break}}});
    let send=|stdin:&mut std::process::ChildStdin,s:&str|->Result<(),String>{writeln!(stdin,"{s}").and_then(|_|stdin.flush()).map_err(|e|e.to_string())};
    let until=|token:&str,timeout:Duration|->Result<(),String>{let end=Instant::now()+timeout;loop{let remaining=end.saturating_duration_since(Instant::now());let line=rx.recv_timeout(remaining).map_err(|_|"引擎初始化超时或进程已退出")?;if line.trim()==token{return Ok(())}}};
    send(&mut stdin,"uci")?;until("uciok",Duration::from_secs(10))?;
    // Relative NNUE filename also works in Windows folders containing Chinese characters.
    send(&mut stdin,"setoption name EvalFile value pikafish.nnue")?;
    send(&mut stdin,&format!("setoption name Threads value {}",config.threads))?;
    send(&mut stdin,&format!("setoption name Hash value {}",config.hash))?;
    send(&mut stdin,"isready")?;until("readyok",Duration::from_secs(15))?;
    send(&mut stdin,"ucinewgame")?;send(&mut stdin,&format!("position fen {fen}"))?;
    let started=Instant::now();send(&mut stdin,&format!("go depth {} movetime {}",config.depth,config.time))?;
    let mut result=Analysis{fen:fen.into(),source:"Pikafish 本机".into(),..Default::default()};
    let deadline=Instant::now()+Duration::from_millis(config.time+10000);
    loop {
        let line=rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).map_err(|_|"引擎计算超时或已退出，请缩短思考时间重试")?;
        if line.starts_with("info "){parse_info(&line,&mut result);}
        if line.starts_with("bestmove ") {
            let best=line.split_whitespace().nth(1).unwrap_or("");
            if position::valid_move(best){result.bestmove=best.into();if result.pv.first().map(String::as_str)!=Some(best){result.pv=vec![best.into()];}}
            result.time=started.elapsed().as_millis() as u64;let _=send(&mut stdin,"quit");return Ok(result);
        }
    }
}
pub fn parse_info(line:&str,r:&mut Analysis){
    let words:Vec<_>=line.split_whitespace().collect();
    if !words.contains(&"pv") {return;}
    for i in 0..words.len(){let next=words.get(i+1).copied().unwrap_or("");match words[i]{
        "depth"=>r.depth=next.parse().unwrap_or(r.depth),
        "score"=>{if let Some(n)=words.get(i+2).and_then(|s|s.parse::<i32>().ok()){if next=="cp"{r.score=n;r.mate=None;}else if next=="mate"{r.mate=Some(n);r.score=if n>=0{30000}else{-30000};}}},
        "pv"=>{r.pv=words[i+1..].iter().take_while(|m|position::valid_move(m)).map(|s|s.to_string()).collect();break;},_=>{}}
    }
}
pub fn analyze(libs:&Path,fen:&str,settings:&Settings)->Result<Analysis,String>{
    position::parse(fen)?;settings.validate()?;
    if settings.cloud {if let Ok(result)=cloud(fen,settings.cloud_timeout){return Ok(result);}}
    let mut result=local(libs,fen,settings)?;
    if settings.cloud{result.warning=Some("云库未命中或不可用，已使用本机引擎计算。".into());}Ok(result)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn info_parser(){let mut r=Analysis::default();parse_info("info depth 18 score cp -34 nodes 10 pv b2e2 h9g7",&mut r);assert_eq!(r.score,-34);assert_eq!(r.depth,18);assert_eq!(r.pv.len(),2);parse_info("info depth 19 score mate 3 pv b2e2",&mut r);assert_eq!(r.mate,Some(3));parse_info("info depth 20 score cp 80 pv b2e2",&mut r);assert_eq!(r.mate,None);}
    #[test] fn engine_smoke(){let libs=Path::new(env!("CARGO_MANIFEST_DIR")).join("../libs");let s=Settings{depth:8,time:500,threads:1,hash:32,cloud:false,cloud_timeout:1};let r=analyze(&libs,"rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1",&s).unwrap();assert!(position::valid_move(&r.bestmove));assert!(!r.pv.is_empty());}
}
