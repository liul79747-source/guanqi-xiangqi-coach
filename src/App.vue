<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { NConfigProvider, NButton, NIcon, NModal, NInput, NInputNumber, NSwitch, NSelect, NSpin, zhCN, dateZhCN } from 'naive-ui'
import { GridOutline, ScanOutline, SettingsOutline, BookOutline, ArrowBackOutline, SwapVerticalOutline, PlayOutline, PauseOutline, RefreshOutline, CopyOutline, DownloadOutline, ChevronBackOutline, ChevronForwardOutline, CloseOutline, CheckmarkCircleOutline, AlertCircleOutline, OpenOutline, AddOutline } from '@vicons/ionicons5'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import ChessBoard from './components/ChessBoard.vue'
import { START_FEN, parseFen, toFen, names, clone, sideOf, coords, legalTargets, applyMove, notation, pvNotations, validatePosition, explainMove, inCheck } from './chess'
import * as api from './api'
import { defaults, type EngineSettings, type Analysis, type Resources, type Side, type WindowInfo } from './types'
const theme={common:{primaryColor:'#28634f',primaryColorHover:'#367961',primaryColorPressed:'#1b493a',borderRadius:'8px',fontFamily:'"Microsoft YaHei", "PingFang SC", sans-serif'},Button:{fontWeight:'500'}}
const position=ref(parseFen(START_FEN)),flipped=ref(false),selected=ref(''),lastMove=ref(''),result=ref<Analysis|null>(null)
const settings=ref<EngineSettings>({...defaults}),health=ref<Resources|null>(null),error=ref(''),notice=ref(''),busy=ref(false)
const page=ref<'study'|'capture'|'guide'>('study'),editing=ref(false),palette=ref('P'),autoAnalyze=ref(false),showArrow=ref(true)
const settingsOpen=ref(false),fenOpen=ref(false),fenText=ref(''),windowsOpen=ref(false),windowLoading=ref(false)
const windows=ref<WindowInfo[]>([]),windowId=ref<number|null>(null),watching=ref(false),capturing=ref(false),confidence=ref<number|null>(null)
const watchSide=ref<Side>('w'),history=ref<{fen:string;move:string;label:string}[]>([{fen:START_FEN,move:'',label:'初始局面'}]),cursor=ref(0)
const fen=computed(()=>toFen(position.value.board,position.value.side)),targets=computed(()=>selected.value&&!editing.value?legalTargets(position.value.board,position.value.side,selected.value):[])
const currentResult=computed(()=>result.value?.fen===fen.value?result.value:null)
const pv=computed(()=>currentResult.value?pvNotations(position.value,currentResult.value.pv):[])
const bestLabel=computed(()=>currentResult.value?.bestmove?notation(position.value.board,currentResult.value.bestmove):'等待分析')
const activeWindow=computed(()=>windows.value.find(w=>w.id===windowId.value))
const score=computed(()=>{const r=currentResult.value;if(!r)return '—';if(r.mate!==null)return `杀 ${Math.abs(r.mate)}`;return (r.score>=0?'+':'')+(r.score/100).toFixed(2)})
const evaluation=computed(()=>{const r=currentResult.value;if(!r)return '分析后显示局面评估';if(r.mate!==null)return r.mate>0?'当前行棋方有杀棋':'当前行棋方面临杀棋';if(Math.abs(r.score)<60)return '双方形势接近';return `${r.score>0?'当前行棋方':'对方'}${Math.abs(r.score)>200?'明显占优':'略占优势'}`})
const advantage=computed(()=>currentResult.value?50+45*Math.tanh(currentResult.value.score/500):50)
let revision=0, inFlight=false, pendingAnalysis=false, disposed=false, timer:ReturnType<typeof setTimeout>|undefined,toastTimer:ReturnType<typeof setTimeout>|undefined
const unlisteners:UnlistenFn[]=[]
function toast(text:string){notice.value=text;clearTimeout(toastTimer);toastTimer=setTimeout(()=>notice.value='',3500)}
function fail(e:unknown){error.value=e instanceof Error?e.message:String(e)}
function invalidate(){revision++;pendingAnalysis=false;busy.value=false;result.value=null;selected.value='';error.value='';clearTimeout(timer)}
function saveSession(){try{localStorage.setItem('guanqi-session',JSON.stringify({history:history.value,cursor:cursor.value}));}catch{}}
function record(move='',label='编辑局面'){history.value=history.value.slice(0,cursor.value+1);history.value.push({fen:fen.value,move,label});cursor.value=history.value.length-1;saveSession()}
function scheduleAnalysis(){if(autoAnalyze.value&&!editing.value){clearTimeout(timer);timer=setTimeout(runAnalysis,300)}}
async function runAnalysis(){
  if(disposed)return
  if(inFlight){pendingAnalysis=true;busy.value=true;return}
  const invalid=validatePosition(position.value.board);if(invalid){error.value=invalid;return}
  if(inCheck(position.value.board,position.value.side==='w'?'b':'w')){error.value='非行棋方正被将军，请检查行棋方或棋子位置。';return}
  const id=++revision,targetFen=fen.value;busy.value=true;inFlight=true;error.value=''
  try{const answer=await api.analyze(targetFen,{...settings.value});if(id===revision&&targetFen===fen.value){result.value=answer;if(!answer.bestmove)toast('当前行棋方已无合法走法。')}}catch(e){if(id===revision)fail(e)}finally{inFlight=false;busy.value=false;if(pendingAnalysis&&!disposed){pendingAnalysis=false;void runAnalysis()}}
}
function movePiece(move:string){try{const label=notation(position.value.board,move);position.value=applyMove(position.value,move);invalidate();lastMove.value=move;record(move,label);scheduleAnalysis()}catch(e){fail(e)}}
function selectSquare(sq:string){
  if(watching.value){toast('请先暂停窗口识别，再操作棋盘。');return}
  const [r,c]=coords(sq),piece=position.value.board[r][c]
  if(editing.value){const board=clone(position.value.board);board[r][c]=palette.value;position.value={...position.value,board};invalidate();record();return}
  if(piece&&sideOf(piece)===position.value.side){selected.value=selected.value===sq?'':sq;return}
  if(selected.value)movePiece(selected.value+sq)
}
function jump(index:number){if(index<0||index>=history.value.length||watching.value)return;invalidate();cursor.value=index;position.value=parseFen(history.value[index].fen);lastMove.value=history.value[index].move;saveSession();scheduleAnalysis()}
function reset(){if(watching.value)return;invalidate();position.value=parseFen(START_FEN);lastMove.value='';record('','重新开局');editing.value=false;toast('已回到初始局面')}
function changeSide(side:Side){if(watching.value)return;invalidate();position.value={...position.value,side};record('','切换行棋方');scheduleAnalysis()}
function toggleEdit(){if(watching.value)return;editing.value=!editing.value;selected.value='';if(!editing.value){const invalid=validatePosition(position.value.board);if(invalid)error.value=invalid;else scheduleAnalysis()}}
function importFen(){try{const next=parseFen(fenText.value);const invalid=validatePosition(next.board);if(invalid)throw new Error(invalid);invalidate();position.value=next;lastMove.value='';record('','导入 FEN');fenOpen.value=false;scheduleAnalysis();toast('局面已导入')}catch(e){fail(e)}}
async function copyFen(){try{await navigator.clipboard.writeText(fen.value);toast('FEN 已复制')}catch{fenText.value=fen.value;fenOpen.value=true;toast('请在文本框中手动复制')}}
function exportGame(){const content={name:'观棋研习记录',version:1,createdAt:new Date().toISOString(),fen:fen.value,history:history.value.slice(0,cursor.value+1),analysis:currentResult.value};const url=URL.createObjectURL(new Blob([JSON.stringify(content,null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=`观棋-${new Date().toISOString().slice(0,10)}.json`;a.click();URL.revokeObjectURL(url);toast('研习记录已导出')}
async function openWindows(){windowsOpen.value=true;windowLoading.value=true;try{windows.value=await api.listWindows()}catch(e){fail(e)}finally{windowLoading.value=false}}
async function captureOnce(){if(windowId.value===null)return;capturing.value=true;error.value='';try{const data=await api.capture(windowId.value,watchSide.value);invalidate();position.value=parseFen(data.fen);confidence.value=data.confidence;lastMove.value='';record('','窗口识别');await runAnalysis()}catch(e){fail(e)}finally{capturing.value=false}}
async function toggleWatch(){try{if(watching.value){await api.stopWatch();watching.value=false;toast('已暂停识别')}else{if(windowId.value===null){await openWindows();return}editing.value=false;await api.startWatch(windowId.value,watchSide.value);watching.value=true;toast('正在识别窗口；首次行棋方使用你选择的红/黑方')}}catch(e){fail(e)}}
watch(settings,()=>{try{localStorage.setItem('guanqi-settings',JSON.stringify(settings.value))}catch{}},{deep:true})
onMounted(async()=>{
  try{const saved=JSON.parse(localStorage.getItem('guanqi-settings')||'null');if(saved){const wasPreviousDefault=saved.depth===20&&saved.time===2000&&saved.threads===2&&saved.hash===128&&saved.cloud===true&&saved.cloudTimeout===2;settings.value={...defaults,...saved};if(wasPreviousDefault)settings.value={...settings.value,depth:defaults.depth,time:defaults.time,cloudTimeout:defaults.cloudTimeout}}}catch{}
  try{const saved=JSON.parse(localStorage.getItem('guanqi-session')||'null');if(saved?.history?.length&&saved.history[saved.cursor]){const pos=parseFen(saved.history[saved.cursor].fen);history.value=saved.history;cursor.value=saved.cursor;position.value=pos;lastMove.value=saved.history[saved.cursor].move}}catch{}
  try{health.value=await api.resources()}catch(e){fail(e)}
  if(api.desktop){
    unlisteners.push(await listen<{fen:string;confidence:number;recoveredPlies?:number;warning?:string}>('board-update',({payload})=>{if(!watching.value)return;invalidate();position.value=parseFen(payload.fen);confidence.value=payload.confidence;lastMove.value='';record('','窗口识别');error.value=payload.warning||'';if(payload.recoveredPlies)toast(`已自动补回 ${payload.recoveredPlies} 手；请核对棋谱`);void runAnalysis()}))
    unlisteners.push(await listen<string>('capture-error',({payload})=>{error.value=payload;if(!payload)toast('窗口画面已恢复，继续识别')}))
  }
})
onUnmounted(()=>{disposed=true;clearTimeout(timer);clearTimeout(toastTimer);unlisteners.forEach(fn=>fn());if(watching.value)void api.stopWatch()})
</script>
<template>
<n-config-provider :theme-overrides="theme" :locale="zhCN" :date-locale="dateZhCN">
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand"><div class="brand-seal">棋</div><div><strong>观棋</strong><span>每一步，都有思路</span></div></div>
      <div class="nav-caption">我的研习室</div>
      <nav aria-label="主导航">
        <button :class="{active:page==='study'}" @click="page='study'"><n-icon :component="GridOutline"/>棋局研习<span class="nav-dot"/></button>
        <button :class="{active:page==='capture'}" @click="page='capture'"><n-icon :component="ScanOutline"/>窗口识别</button>
        <button :class="{active:page==='guide'}" @click="page='guide'"><n-icon :component="BookOutline"/>使用指南</button>
      </nav>
      <div class="sidebar-bottom"><div class="quiet-quote">观一局棋，<br/>得一分进益。</div><button class="settings-nav" @click="settingsOpen=true"><n-icon :component="SettingsOutline"/>分析设置<span>↗</span></button><div class="version">观棋研习室 <span>v1.0</span></div></div>
    </aside>
    <div class="workspace">
      <header class="topbar"><div class="breadcrumb">研习室 <span>/</span> {{page==='capture'?'窗口识别':page==='guide'?'使用指南':'棋局研习'}}</div><div class="top-status"><span :class="['status-dot',{offline:!health?.engine}]"/>{{health?.engine?'Pikafish 已就绪':'检查引擎资源'}}<span class="mode-badge">{{api.desktop?'桌面版':'本地浏览器版'}}</span></div></header>
      <main>
        <div class="page-heading"><div><div class="eyebrow">XIANGQI STUDIO</div><h1>{{page==='capture'?'让棋局，同步眼前':page==='guide'?'从看懂一步，到读懂一局':'落子之前，多想一步'}}</h1><p>{{page==='capture'?'连接棋盘窗口，识别局面后获取走法建议。':page==='guide'?'认识你的棋盘、引擎与研习工具。':'摆一盘棋，推演变化，和引擎一起打磨你的思路。'}}</p></div><n-button v-if="page!=='guide'" secondary @click="exportGame"><template #icon><n-icon :component="DownloadOutline"/></template>导出棋局</n-button></div>
        <div v-if="error" class="alert error" role="alert"><n-icon :component="AlertCircleOutline"/><span>{{error}}</span><button aria-label="关闭错误" @click="error=''">×</button></div>
        <template v-if="page==='guide'">
          <div class="guide-grid"><article class="guide-card"><span class="step-number">01</span><h2>从一个局面开始</h2><p>点击自己的棋子，再点击绿色标记的位置走棋。可以导入 FEN，或打开“摆棋模式”自由放置棋子。红方在下，点击“翻转”切换视角。</p></article><article class="guide-card"><span class="step-number">02</span><h2>理解推荐走法</h2><p>点击“分析局面”查询云库，未命中时由本机 Pikafish 计算。评分以当前行棋方为视角，正数有利、负数不利。“走出此步”会在研习棋盘执行推荐着法。</p></article><article class="guide-card"><span class="step-number">03</span><h2>连接一个棋盘窗口</h2><p>桌面版可以选择窗口并识别棋盘。保持目标窗口可见、不要最小化，选择正确的首次行棋方。漏帧导致无法判断时，程序会自动重新同步并继续识别，同时提示你核对行棋方；画面无法识别时会在识别恢复后继续。</p></article><article class="guide-card"><span class="step-number">04</span><h2>保存你的研习过程</h2><p>棋谱会自动保存在当前设备。点击棋谱条目可回看局面，再走棋会产生新的分支。导出文件包含当前 FEN、走棋记录和最近一次分析，方便留存。</p></article></div>
          <div class="guide-note"><h3>使用与资源</h3><p>适用于个人学习、复盘和练习。参考项目：atopx/chessboard（Apache-2.0）；计算引擎：Pikafish（GPL-3.0）。识别结果需要人工核对，学习提示按棋子和着法规则生成，不是引擎对策略的自然语言解释。</p><p>资源路径：{{health?.resourceDir||'等待本地服务'}}</p></div>
        </template>
        <template v-else>
          <section v-if="page==='capture'" class="capture-panel"><div><n-icon :component="ScanOutline"/><div><strong>{{activeWindow?.title||'选择要识别的棋盘窗口'}}</strong><p>{{api.desktop?'首次行棋方请手动选择，后续按识别到的移动更新。':'窗口截图需打开 Tauri 桌面版；此处仍可手动摆棋和分析。'}}</p></div></div><div class="capture-actions"><n-select v-model:value="watchSide" :disabled="watching" :options="[{label:'首次红方走',value:'w'},{label:'首次黑方走',value:'b'}]" style="width:135px"/><n-button :disabled="!api.desktop||watching" @click="openWindows">选择窗口</n-button><n-button :disabled="!api.desktop||windowId===null||watching" :loading="capturing" @click="captureOnce">识别一次</n-button><n-button type="primary" :disabled="!api.desktop" @click="toggleWatch">{{watching?'暂停识别':'连续识别'}}</n-button></div></section>
          <div class="study-grid">
            <section class="board-panel">
              <div class="panel-heading"><div class="board-title"><span class="mini-seal">弈</span><h2>{{editing?'自由摆棋':'推演棋盘'}}</h2><span class="subtle">{{history[cursor]?.label==='窗口识别'?'来自窗口':'自由研习'}}</span></div><div class="turn-pill"><i :class="position.side==='w'?'red':'black'"/>{{position.side==='w'?'红方':'黑方'}}行棋</div></div>
              <div class="board-stage"><ChessBoard :board="position.board" :flipped="flipped" :selected="selected" :targets="targets" :arrow="showArrow?currentResult?.bestmove||'':''" :last-move="lastMove" @select="selectSquare"/></div>
              <div v-if="editing" class="palette"><button v-for="p in ['K','A','B','N','R','C','P','k','a','b','n','r','c','p','']" :key="p" :class="{chosen:palette===p,red:p&&sideOf(p)==='w'}" :aria-label="p?'放置'+names[p]:'移除棋子'" @click="palette=p">{{p?names[p]:'×'}}</button><span>选棋子，再点击棋盘放置</span></div>
              <div class="board-toolbar"><div><n-button quaternary size="small" @click="flipped=!flipped"><template #icon><n-icon :component="SwapVerticalOutline"/></template>翻转</n-button><n-button quaternary size="small" :disabled="cursor===0||watching" @click="jump(cursor-1)"><template #icon><n-icon :component="ArrowBackOutline"/></template>悔棋</n-button><n-button quaternary size="small" :disabled="watching" @click="reset"><template #icon><n-icon :component="RefreshOutline"/></template>开局</n-button></div><n-button size="small" :type="editing?'primary':'default'" :disabled="watching" @click="toggleEdit">{{editing?'完成摆棋':'摆棋模式'}}</n-button></div>
              <div class="board-foot"><span><span class="status-dot"/>{{watching?'正在同步窗口':editing?'编辑中 · 可自由放置棋子':'点击棋子，开始推演'}}</span><span>{{confidence!==null?`识别置信度 ${Math.round(confidence*100)}%`:'10 × 9 · 中国象棋'}}</span></div>
            </section>
            <div class="analysis-column">
              <section class="analysis-card"><div class="panel-heading"><h2>局面分析</h2><span class="small-tag">{{currentResult?.source||'Pikafish'}}</span></div><div class="eval-area"><span class="muted-label">{{position.side==='w'?'红方':'黑方'}}视角评估</span><div class="score-row"><strong>{{score}}</strong><span>{{evaluation}}</span></div><div class="eval-bar"><div :style="{width:advantage+'%'}"/></div><div class="eval-legend"><span>对方有利</span><span>己方有利</span></div></div>
                <div class="best-move"><div class="muted-label"><span class="recommend-dot"/>推荐着法</div><div class="move-row"><strong>{{busy?'正在思考…':bestLabel}}</strong><span v-if="currentResult?.bestmove">{{currentResult.bestmove}}</span></div><div class="search-meta"><span>深度 <b>{{currentResult?.depth||'—'}}</b></span><span>用时 <b>{{currentResult?(currentResult.time/1000).toFixed(1)+' s':'—'}}</b></span><span>{{currentResult?'已完成':'就绪'}}</span></div></div>
                <div class="analysis-actions"><n-button type="primary" size="large" block :loading="busy" :disabled="editing" @click="runAnalysis"><template #icon><n-icon :component="PlayOutline"/></template>{{busy?'正在分析':'分析局面'}}</n-button><n-button block :disabled="!currentResult?.bestmove||watching||editing||busy" @click="movePiece(currentResult!.bestmove)">走出此步 <span class="button-arrow">↗</span></n-button></div><div class="auto-row"><span>走棋后自动分析</span><n-switch v-model:value="autoAnalyze" size="small" :disabled="editing"/></div>
              </section>
              <section class="variation-card"><div class="panel-heading"><h2>主变化</h2><span class="subtle">{{pv.length?pv.length+' 个半回合':'等待引擎'}}</span></div><div v-if="pv.length" class="pv-list"><span v-for="(move,i) in pv" :key="i"><small>{{i+1}}.</small>{{move}}</span></div><div v-else class="empty-variation"><span>⌁</span><p>好的着法，也经得起推演</p><small>分析后在这里查看后续变化</small></div><div v-if="currentResult?.bestmove" class="learning-note"><span>研习提示</span><p>{{explainMove(position.board,currentResult.bestmove)}}</p></div><p v-if="currentResult?.warning" class="cloud-warning">{{currentResult.warning}}</p></section>
            </div>
            <section class="record-panel"><div class="panel-heading"><h2>棋谱记录</h2><span class="count-tag">{{cursor}}</span></div><div class="record-list"><button v-for="(entry,i) in history" :key="i" :class="{current:cursor===i}" :disabled="watching" @click="jump(i)"><span>{{String(i).padStart(2,'0')}}</span><strong>{{entry.label}}</strong><i v-if="cursor===i"/></button></div><div class="record-controls"><n-button quaternary size="small" :disabled="cursor===0||watching" @click="jump(0)">|‹</n-button><n-button quaternary size="small" :disabled="cursor===0||watching" aria-label="上一步" @click="jump(cursor-1)"><n-icon :component="ChevronBackOutline"/></n-button><n-button quaternary size="small" :disabled="cursor>=history.length-1||watching" aria-label="下一步" @click="jump(cursor+1)"><n-icon :component="ChevronForwardOutline"/></n-button><n-button quaternary size="small" :disabled="cursor>=history.length-1||watching" @click="jump(history.length-1)">›|</n-button></div><div class="position-tools"><span class="muted-label">局面工具</span><label>行棋方<n-select :value="position.side" :disabled="watching" :options="[{label:'红方',value:'w'},{label:'黑方',value:'b'}]" size="small" @update:value="changeSide"/></label><label>推荐箭头<n-switch v-model:value="showArrow" size="small"/></label><n-button size="small" block :disabled="watching" @click="fenText=fen;fenOpen=true"><template #icon><n-icon :component="OpenOutline"/></template>导入 FEN</n-button><n-button size="small" block @click="copyFen"><template #icon><n-icon :component="CopyOutline"/></template>复制当前 FEN</n-button></div><div class="record-tip">一局一得<br/><span>记录思考，比记住答案更重要。</span></div></section>
          </div>
          <footer class="workspace-footer"><span>本地研习 · 自动保存棋谱</span><span>云库优先 / 本机引擎后备 <i>·</i> {{settings.depth}} 层 / {{settings.time/1000}} 秒</span></footer>
        </template>
      </main>
    </div>
    <div v-if="notice" class="toast" role="status"><n-icon :component="CheckmarkCircleOutline"/>{{notice}}</div>
  </div>
  <n-modal v-model:show="settingsOpen" preset="card" title="分析设置" class="settings-modal" style="width:500px;max-width:92vw"><p class="modal-intro">设置自动保存，下一次分析时生效。</p><div class="setting-row"><label>搜索深度<span>同时受到思考时间限制</span></label><n-input-number v-model:value="settings.depth" :min="1" :max="60" :precision="0" :update-value-on-input="false"/></div><div class="setting-row"><label>思考时间（毫秒）</label><n-input-number v-model:value="settings.time" :min="100" :max="30000" :step="500" :precision="0"/></div><div class="setting-row"><label>引擎线程</label><n-input-number v-model:value="settings.threads" :min="1" :max="16" :precision="0"/></div><div class="setting-row"><label>置换表内存（MB）</label><n-input-number v-model:value="settings.hash" :min="16" :max="1024" :step="64" :precision="0"/></div><div class="setting-row"><label>优先查询 ChessDB 云库<span>查询时仅发送当前 FEN 局面</span></label><n-switch v-model:value="settings.cloud"/></div><div class="setting-row"><label>云库超时（秒）</label><n-input-number v-model:value="settings.cloudTimeout" :min="1" :max="10" :precision="0"/></div><div class="resource-check"><span :class="{ready:health?.engine}">● 引擎 {{health?.engine?'就绪':'缺失'}}</span><span :class="{ready:health?.model}">● 模型 {{health?.model?'就绪':'缺失'}}</span><span :class="{ready:health?.runtime}">● 运行库 {{health?.runtime?'就绪':'缺失'}}</span></div><n-button block secondary @click="settings={...defaults};toast('已恢复默认设置')">恢复默认设置</n-button></n-modal>
  <n-modal v-model:show="fenOpen" preset="card" title="导入局面 / FEN" style="width:560px;max-width:92vw"><p class="modal-intro">粘贴中国象棋 FEN。w 为红方走，b 为黑方走。</p><n-input v-model:value="fenText" type="textarea" :rows="4" placeholder="rnbakabnr/9/… w - - 0 1"/><p v-if="error" class="form-error">{{error}}</p><div class="modal-actions"><n-button @click="fenOpen=false">取消</n-button><n-button type="primary" @click="importFen">导入局面</n-button></div></n-modal>
  <n-modal v-model:show="windowsOpen" preset="card" title="选择棋盘窗口" style="width:620px;max-width:92vw"><p class="modal-intro">选择完整显示棋盘的窗口。目标窗口不要最小化。</p><n-spin :show="windowLoading"><div class="window-list"><button v-for="win in windows" :key="win.id" :class="{chosen:windowId===win.id}" @click="windowId=win.id;windowsOpen=false;toast('已选择窗口')"><n-icon :component="ScanOutline"/><div><strong>{{win.title}}</strong><span>{{win.app}}</span></div></button><p v-if="!windows.length">暂未发现可用窗口。</p></div></n-spin><n-button @click="openWindows">刷新列表</n-button></n-modal>
</n-config-provider>
</template>
