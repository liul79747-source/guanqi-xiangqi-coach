// Inference adapted from atopx/chessboard (Apache-2.0), rewritten 2026-10-02:
// runtime-loaded model, checked tensor shapes, class-aware NMS, board-relative coordinates.
use std::{path::Path, sync::{Mutex, OnceLock}};
use ndarray::Array4;
use ort::{inputs, session::{Session, builder::GraphOptimizationLevel}};
use xcap::image::{DynamicImage, ImageBuffer, Rgba, imageops::FilterType};
use crate::position::{self,Board};
type Image=ImageBuffer<Rgba<u8>,Vec<u8>>;
static MODEL: OnceLock<Mutex<Session>>=OnceLock::new();
static INIT:Mutex<()>=Mutex::new(());
const LABELS:[char;15]=['n','b','a','k','r','c','p','R','N','A','K','B','C','P','0'];
#[derive(Clone,Debug)] struct Detection{x:f32,y:f32,w:f32,h:f32,score:f32,label:char}
pub fn init(libs:&Path)->Result<(),String>{
    let _guard=INIT.lock().map_err(|_|"模型初始化锁异常")?;
    if MODEL.get().is_some(){return Ok(());}
    let runtime=libs.join("runtime/onnxruntime.dll");let model=libs.join("models/board.onnx");
    if !runtime.is_file() || !model.is_file(){return Err("缺少棋盘识别模型或 ONNX 运行库，请检查 libs/models 和 libs/runtime".into());}
    ort::init_from(runtime.to_string_lossy()).commit().map_err(|e|format!("ONNX 初始化失败：{e}"))?;
    let session=Session::builder().map_err(|e|e.to_string())?.with_optimization_level(GraphOptimizationLevel::Level3).map_err(|e|e.to_string())?.with_intra_threads(2).map_err(|e|e.to_string())?.commit_from_file(model).map_err(|e|format!("模型加载失败：{e}"))?;
    MODEL.set(Mutex::new(session)).map_err(|_|"模型重复初始化".to_string())?;Ok(())
}
fn predict(image:&Image)->Result<Vec<Detection>,String>{
    let resized=DynamicImage::ImageRgba8(image.clone()).resize_exact(640,640,FilterType::Triangle).to_rgb8();
    let mut input=Array4::<f32>::zeros((1,3,640,640));
    for (x,y,p) in resized.enumerate_pixels(){for c in 0..3{input[[0,c,y as usize,x as usize]]=p[c] as f32/255.;}}
    let session=MODEL.get().ok_or("模型未初始化")?.lock().map_err(|_|"模型锁异常")?;
    let output=session.run(inputs![input.view()].map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let tensor=output[0].try_extract_tensor::<f32>().map_err(|e|e.to_string())?;
    let shape=tensor.shape();
    if shape.len()!=3 || shape[0]!=1 {return Err(format!("不支持的模型输出维度：{shape:?}"));}
    let channels_first=shape[1]==19 || shape[1]==20;
    let (count,channels)=if channels_first{(shape[2],shape[1])}else{(shape[1],shape[2])};
    if channels!=19 && channels!=20{return Err("模型需要含 15 类棋子/棋盘标签".into());}
    let at=|i:usize,c:usize|if channels_first{tensor[[0,c,i]]}else{tensor[[0,i,c]]};
    let mut detections=Vec::new();
    for i in 0..count {
        let start=if channels==20{5}else{4};let mut label=0;let mut prob=0.;
        for c in 0..15{let value=at(i,start+c);if value>prob{prob=value;label=c;}}
        let score=prob*if channels==20{at(i,4)}else{1.};
        if score>=0.65 {let d=Detection{x:at(i,0),y:at(i,1),w:at(i,2),h:at(i,3),score,label:LABELS[label]};if [d.x,d.y,d.w,d.h,score].iter().all(|n|n.is_finite())&&d.w>0.&&d.h>0.{detections.push(d);}}
    }
    detections.sort_by(|a,b|b.score.total_cmp(&a.score));let mut accepted:Vec<Detection>=Vec::new();
    for d in detections {if accepted.iter().any(|a|{if (a.label=='0')!=(d.label=='0'){return false;}let w=((a.x+a.w/2.).min(d.x+d.w/2.)-(a.x-a.w/2.).max(d.x-d.w/2.)).max(0.);let h=((a.y+a.h/2.).min(d.y+d.h/2.)-(a.y-a.h/2.).max(d.y-d.h/2.)).max(0.);w*h/(a.w*a.h+d.w*d.h-w*h)>0.45}){continue;}accepted.push(d);}
    Ok(accepted)
}
pub fn recognize(image:Image)->Result<(Board,f32),String>{
    let initial=predict(&image)?;let bounds=initial.iter().find(|d|d.label=='0').ok_or("未识别到棋盘，请保持目标窗口可见且棋盘完整")?;
    let sx=image.width() as f32/640.;let sy=image.height() as f32/640.;
    let x=((bounds.x-bounds.w/2.-bounds.w/18.)*sx).max(0.) as u32;let y=((bounds.y-bounds.h/2.-bounds.h/20.)*sy).max(0.) as u32;
    let end_x=((bounds.x+bounds.w/2.+bounds.w/18.)*sx).min(image.width() as f32) as u32;let end_y=((bounds.y+bounds.h/2.+bounds.h/20.)*sy).min(image.height() as f32) as u32;
    if end_x<=x || end_y<=y{return Err("棋盘裁剪范围无效".into());}
    let cropped=xcap::image::imageops::crop_imm(&image,x,y,end_x-x,end_y-y).to_image();let detections=predict(&cropped)?;
    let mut board=[[' ';9];10];let mut scores=Vec::new();
    // Expanded crop includes half a cell outside the 8×9 grid, matching the reference model.
    for d in detections.iter().filter(|d|d.label!='0'){
        let c=(d.x/(640./9.)).floor() as i32;let r=(d.y/(640./10.)).floor() as i32;
        if !(0..9).contains(&c)||!(0..10).contains(&r){continue;}
        if board[r as usize][c as usize]!=' ' {return Err("同一格识别出多个棋子，请调整窗口大小".into());}
        board[r as usize][c as usize]=d.label;scores.push(d.score);
    }
    let bottom_king=(7..10).any(|r|(3..6).any(|c|board[r][c]=='k'));
    if bottom_king{board.reverse();for row in &mut board{row.reverse();}}
    position::validate(&board)?;
    let confidence=scores.iter().copied().fold(1.0_f32,f32::min);
    Ok((board,confidence))
}
#[cfg(test)] mod tests{
    use super::*;
    #[test] fn model_loads(){init(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../libs")).unwrap();}
    #[test] fn recognizes_reference_board(){
        let root=Path::new(env!("CARGO_MANIFEST_DIR"));init(&root.join("../libs")).unwrap();
        let image=xcap::image::open(root.join("../tests/fixtures/reference.png")).unwrap().to_rgba8();
        let crop=xcap::image::imageops::crop_imm(&image,0,0,1015,1033).to_image();
        let (board,confidence)=recognize(crop).unwrap();
        assert_eq!(board[0][3],'k');assert_eq!(board[9][4],'K');assert_eq!(board[1][5],'C');
        assert!(confidence>=0.65);assert_eq!(board.iter().flatten().filter(|&&p|p!=' ').count(),22);
        assert_eq!(position::fen(&board,'w'),"1rbk2R2/4RC3/2n6/p3p3p/2p6/9/P1P1P3P/2N5N/9/2BAKAB2 w - - 0 1");
    }
}
