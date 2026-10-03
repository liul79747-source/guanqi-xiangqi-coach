import { invoke, isTauri } from '@tauri-apps/api/core'
import type { Analysis, EngineSettings, Resources, Side, WindowInfo } from './types'
export const desktop = isTauri()
async function call<T>(command: string, args: Record<string,unknown> = {}): Promise<T> {
  if(desktop) return invoke<T>(command,args)
  const response=await fetch(`/api/${command}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(args)})
  if(!response.ok){const error=await response.json().catch(()=>({error:'本地分析服务未启动，请使用“启动观棋.cmd”。'}));throw new Error(error.error||'请求失败')}
  return response.json()
}
export const resources=()=>call<Resources>('resources')
export const analyze=(fen:string,settings:EngineSettings)=>call<Analysis>('analyze',{fen,settings})
export const listWindows=()=>call<WindowInfo[]>('list_windows')
export const capture=(windowId:number,side:Side)=>call<{fen:string;confidence:number}>('capture',{windowId,side})
export const startWatch=(windowId:number,side:Side)=>call<void>('start_watch',{windowId,side})
export const stopWatch=()=>call<void>('stop_watch')
