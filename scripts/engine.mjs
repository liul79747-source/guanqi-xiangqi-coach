import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { parseFen, validatePosition, legalMove, inCheck, opposite } from '../src/chess.ts'
export const libs=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../libs')
const validMove=m=>/^[a-i][0-9][a-i][0-9]$/.test(m)
export function validateSettings(s){
  for(const [key,min,max] of [['depth',1,60],['time',100,30000],['threads',1,16],['hash',16,1024],['cloudTimeout',1,10]]) if(!Number.isInteger(s?.[key])||s[key]<min||s[key]>max)throw new Error(`分析参数 ${key} 超出有效范围`)
  if(typeof s.cloud!=='boolean')throw new Error('云库设置无效')
}
export function validateFen(fen){
  if(typeof fen!=='string'||fen.length>180||/[\r\n]/.test(fen))throw new Error('FEN 格式错误')
  const pos=parseFen(fen),invalid=validatePosition(pos.board)
  if(invalid)throw new Error(invalid)
  if(inCheck(pos.board,opposite(pos.side)))throw new Error('非行棋方正被将军，请校正局面或行棋方')
  return pos
}
export function parseInfo(line,result){
  const t=line.trim().split(/\s+/),pv=t.indexOf('pv');if(pv<0)return
  const depth=t.indexOf('depth'),score=t.indexOf('score')
  if(depth>=0&&Number.isFinite(Number(t[depth+1])))result.depth=Number(t[depth+1])
  if(score>=0){const n=Number(t[score+2]);if(Number.isFinite(n)){result.mate=t[score+1]==='mate'?n:null;result.score=result.mate!==null?(n>=0?30000:-30000):n}}
  result.pv=t.slice(pv+1).filter(validMove)
}
export async function queryCloud(fen,timeout){
  const url=new URL('https://www.chessdb.cn/chessdb.php');url.searchParams.set('action','querypv');url.searchParams.set('board',fen)
  const start=performance.now(),response=await fetch(url,{signal:AbortSignal.timeout(timeout*1000)})
  if(!response.ok)throw new Error('云库服务不可用')
  const text=(await response.text()).replace(/\0/g,'').trim(),values=Object.fromEntries(text.split(',').map(s=>{const i=s.indexOf(':');return [s.slice(0,i),s.slice(i+1)]}))
  const pv=(values.pv||'').split('|').filter(validMove)
  if(!pv.length||!Number.isFinite(Number(values.score)))throw new Error('云库未收录')
  return {fen,bestmove:pv[0],pv,score:Number(values.score),mate:null,depth:Number(values.depth)||0,time:Math.round(performance.now()-start),source:'ChessDB 云库'}
}
export function queryEngine(fen,settings){return new Promise((resolve,reject)=>{
  const exe=path.join(libs,'pikafish/pikafish-windows.exe'),nnue=path.join(libs,'pikafish/pikafish.nnue')
  if(!existsSync(exe)||!existsSync(nnue))return reject(new Error('Pikafish 引擎或 NNUE 文件缺失'))
  const child=spawn(exe,[],{cwd:path.dirname(exe),windowsHide:true,stdio:['pipe','pipe','pipe']})
  const lines=createInterface({input:child.stdout}),result={fen,bestmove:'',pv:[],score:0,mate:null,depth:0,time:0,source:'Pikafish 本机'}
  let settled=false,started=performance.now(),phase='uci',stderr=''
  const timer=setTimeout(()=>finish(new Error('引擎超时，请重试或缩短思考时间')),settings.time+25000)
  function finish(error){if(settled)return;settled=true;clearTimeout(timer);lines.close();child.kill();error?reject(error):resolve(result)}
  function send(command){if(!settled&&!child.stdin.destroyed)child.stdin.write(command+'\n')}
  child.stdin.on('error',e=>finish(e));child.on('error',e=>finish(e));child.stderr.on('data',chunk=>stderr=(stderr+chunk.toString()).slice(-1000))
  child.on('exit',code=>{if(!settled)finish(new Error(`引擎提前退出（${code}）${stderr}`))})
  lines.on('line',line=>{
    if(phase==='uci'&&line==='uciok'){
      phase='ready';send('setoption name EvalFile value pikafish.nnue');send(`setoption name Threads value ${settings.threads}`);send(`setoption name Hash value ${settings.hash}`);send('isready')
    }else if(phase==='ready'&&line==='readyok'){
      phase='search';send('ucinewgame');send(`position fen ${fen}`);started=performance.now();send(`go depth ${settings.depth} movetime ${settings.time}`)
    }else if(phase==='search'&&line.startsWith('info '))parseInfo(line,result)
    else if(phase==='search'&&line.startsWith('bestmove ')){
      const best=line.split(/\s+/)[1];result.bestmove=validMove(best)?best:'';if(result.bestmove&&result.pv[0]!==best)result.pv=[best];result.time=Math.round(performance.now()-started);send('quit');finish()
    }
  });send('uci')
})}
export async function analyze(fen,settings){
  const pos=validateFen(fen);validateSettings(settings)
  if(settings.cloud){try{const r=await queryCloud(fen,settings.cloudTimeout);if(legalMove(pos.board,pos.side,r.bestmove))return r}catch{}}
  const result=await queryEngine(fen,settings)
  if(result.bestmove&&!legalMove(pos.board,pos.side,result.bestmove))throw new Error('引擎返回了不适用于当前局面的走法')
  if(settings.cloud)result.warning='云库未命中或不可用，已使用本机引擎计算。'
  return result
}
