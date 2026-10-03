import http from 'node:http'
import { readFile, stat } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { analyze, libs } from './engine.mjs'
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../dist')
const port=Number(process.env.PORT||1422)
let busy=false
const send=(res,status,data)=>{res.writeHead(status,{'Content-Type':'application/json; charset=utf-8','Cache-Control':'no-store'});res.end(JSON.stringify(data))}
const server=http.createServer(async(req,res)=>{
  const allowed=new Set([`127.0.0.1:${port}`,`localhost:${port}`])
  if(!allowed.has(req.headers.host)){send(res,403,{error:'仅允许本机访问'});return}
  const origin=req.headers.origin
  if(origin&&!new Set([`http://127.0.0.1:${port}`,`http://localhost:${port}`,'http://127.0.0.1:1420','http://localhost:1420']).has(origin)){send(res,403,{error:'不允许跨站请求'});return}
  const url=new URL(req.url,'http://127.0.0.1')
  if(url.pathname.startsWith('/api/')){
    if(req.method!=='POST'){send(res,405,{error:'请使用 POST'});return}
    if(!req.headers['content-type']?.startsWith('application/json')){send(res,415,{error:'需要 JSON 请求'});return}
    try{
      let body='';for await(const chunk of req){body+=chunk;if(body.length>8192){send(res,413,{error:'请求过大'});return}}
      const args=JSON.parse(body||'{}'),command=url.pathname.slice(5)
      if(command==='resources'){send(res,200,{engine:existsSync(path.join(libs,'pikafish/pikafish-windows.exe'))&&existsSync(path.join(libs,'pikafish/pikafish.nnue')),model:existsSync(path.join(libs,'models/board.onnx')),runtime:existsSync(path.join(libs,'runtime/onnxruntime.dll')),desktop:false,resourceDir:libs});return}
      if(command==='analyze'){
        if(busy){send(res,409,{error:'上一局面仍在计算，请稍后再试'});return}
        busy=true;try{send(res,200,await analyze(args.fen,args.settings))}finally{busy=false}return
      }
      send(res,400,{error:'窗口识别需要使用 Tauri 桌面版。浏览器版支持完整摆棋和引擎分析。'})
    }catch(e){send(res,400,{error:e.message||'请求失败'})}return
  }
  if(req.method!=='GET'){send(res,405,{error:'Method not allowed'});return}
  try{
    const pathname=decodeURIComponent(url.pathname),target=path.resolve(root,'.'+pathname)
    if(target!==root&&!target.startsWith(root+path.sep)){send(res,403,{error:'无效路径'});return}
    let file=target;try{if(!(await stat(file)).isFile())file=path.join(root,'index.html')}catch{file=path.join(root,'index.html')}
    const content=await readFile(file),types={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8','.svg':'image/svg+xml','.png':'image/png','.ico':'image/x-icon'}
    res.writeHead(200,{'Content-Type':types[path.extname(file)]||'application/octet-stream','X-Content-Type-Options':'nosniff','Cache-Control':'no-cache'});res.end(content)
  }catch{send(res,500,{error:'界面尚未构建，请先运行 npm run build'})}
})
server.on('error',e=>{console.error(e.message);process.exitCode=1})
server.listen(port,'127.0.0.1',()=>console.log(`观棋已启动：http://127.0.0.1:${port}（按 Ctrl+C 退出）`))
