import type { Board, Position, Side } from './types'
export const START_FEN = 'rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1'
export const names: Record<string, string> = { r: '車', n: '馬', b: '象', a: '士', k: '將', c: '砲', p: '卒', R: '俥', N: '傌', B: '相', A: '仕', K: '帥', C: '炮', P: '兵' }
export const sideOf = (p: string): Side => p === p.toUpperCase() ? 'w' : 'b'
export const opposite = (s: Side): Side => s === 'w' ? 'b' : 'w'
export const clone = (b: Board): Board => b.map(r => [...r])
export const square = (r: number, c: number) => String.fromCharCode(97 + c) + (9 - r)
export const coords = (sq: string): [number, number] => [9 - Number(sq[1]), sq.charCodeAt(0) - 97]
export function parseFen(fen: string): Position {
  const parts = fen.trim().split(/\s+/)
  if (!['w', 'b'].includes(parts[1])) throw new Error('FEN 需要指定行棋方：w 为红方，b 为黑方。')
  const rows = parts[0].split('/')
  if (rows.length !== 10) throw new Error('FEN 棋盘应为 10 行。')
  const board = rows.map(row => {
    const cells: string[] = []
    for (let p of row) {
      p = ({ h: 'n', H: 'N', e: 'b', E: 'B' } as Record<string, string>)[p] || p
      if (/^[1-9]$/.test(p)) cells.push(...Array(Number(p)).fill(''))
      else if (names[p]) cells.push(p)
      else throw new Error(`FEN 包含无法识别的棋子：${p}`)
    }
    if (cells.length !== 9) throw new Error('FEN 每行必须恰好包含 9 个位置。')
    return cells
  })
  return { board, side: parts[1] as Side }
}
export function toFen(board: Board, side: Side): string {
  return board.map(row => { let s = '', n = 0; for (const p of row) { if (!p) n++; else { if(n) s += n; s += p; n = 0 } } return s + (n || '') }).join('/') + ` ${side} - - 0 1`
}
const inside = (r: number, c: number) => r >= 0 && r < 10 && c >= 0 && c < 9
function between(board: Board, r: number, c: number, y: number, x: number): number {
  let count = 0; const dr = Math.sign(y-r), dc = Math.sign(x-c)
  for (let i=r+dr,j=c+dc; i!==y || j!==x; i+=dr,j+=dc) if (board[i][j]) count++
  return count
}
function pseudo(board: Board, r: number, c: number, y: number, x: number): boolean {
  if (!inside(y,x) || (r===y && c===x) || !board[r][c]) return false
  const p = board[r][c], side = sideOf(p), target = board[y][x]
  if (target && sideOf(target)===side) return false
  const dy=y-r, dx=x-c, ay=Math.abs(dy), ax=Math.abs(dx)
  const palace = x>=3 && x<=5 && (side==='w' ? y>=7 : y<=2)
  switch(p.toLowerCase()) {
    case 'r': return (dx===0 || dy===0) && between(board,r,c,y,x)===0
    case 'c': return (dx===0 || dy===0) && between(board,r,c,y,x)===(target ? 1 : 0)
    case 'n': return (ax===2 && ay===1 && !board[r][c+Math.sign(dx)]) || (ay===2 && ax===1 && !board[r+Math.sign(dy)][c])
    case 'b': return ax===2 && ay===2 && (side==='w' ? y>=5 : y<=4) && !board[r+dy/2][c+dx/2]
    case 'a': return palace && ax===1 && ay===1
    case 'k': return (palace && ax+ay===1) || (target.toLowerCase()==='k' && dx===0 && between(board,r,c,y,x)===0)
    case 'p': return (dx===0 && dy===(side==='w' ? -1 : 1)) || (dy===0 && ax===1 && (side==='w' ? r<=4 : r>=5))
  }
  return false
}
export function inCheck(board: Board, side: Side): boolean {
  const king = side==='w' ? 'K' : 'k'
  let kr=-1,kc=-1
  board.forEach((row,r)=>row.forEach((p,c)=>{if(p===king){kr=r;kc=c}}))
  if(kr<0) return true
  return board.some((row,r)=>row.some((p,c)=>p && sideOf(p)!==side && pseudo(board,r,c,kr,kc)))
}
export function legalMove(board: Board, side: Side, move: string): boolean {
  if (!/^[a-i][0-9][a-i][0-9]$/.test(move)) return false
  const [r,c]=coords(move.slice(0,2)),[y,x]=coords(move.slice(2))
  if(!board[r][c] || sideOf(board[r][c])!==side || !pseudo(board,r,c,y,x)) return false
  const b=clone(board); b[y][x]=b[r][c]; b[r][c]=''
  return !inCheck(b,side)
}
export function applyMove(position: Position, move: string): Position {
  if(!legalMove(position.board,position.side,move)) throw new Error('这步棋不符合走棋规则，或会使己方被将军。')
  const board=clone(position.board),[r,c]=coords(move.slice(0,2)),[y,x]=coords(move.slice(2)); board[y][x]=board[r][c];board[r][c]=''
  return {board,side:opposite(position.side)}
}
export function legalTargets(board: Board, side: Side, from: string): string[] {
  const targets:string[]=[]; for(let r=0;r<10;r++) for(let c=0;c<9;c++) if(legalMove(board,side,from+square(r,c))) targets.push(square(r,c)); return targets
}
export function validatePosition(board: Board): string | null {
  const limits:Record<string,number>={k:1,a:2,b:2,n:2,r:2,c:2,p:5}
  for(const side of ['w','b'] as Side[]) {
    const counts:Record<string,number>={}
    for(let r=0;r<10;r++) for(let c=0;c<9;c++) {
      const p=board[r][c]; if(!p||sideOf(p)!==side) continue
      const type=p.toLowerCase(); counts[type]=(counts[type]||0)+1
      if((type==='k'||type==='a') && !(c>=3&&c<=5&&(side==='w'?r>=7:r<=2))) return '将帅和士仕必须在本方九宫内。'
      if(type==='b' && (side==='w'?r<5:r>4)) return '象相不能过河。'
      if(type==='p' && (side==='w'?r>6:r<3)) return '兵卒不能出现在己方初始兵线后方。'
    }
    if(counts.k!==1) return '红帅和黑将都必须有且仅有一个。'
    for(const [p,n] of Object.entries(counts)) if(n>limits[p]) return '棋子数量超出象棋规则允许的上限。'
  }
  if(inCheck(board,'w') && inCheck(board,'b')) return '双方同时被将军或将帅照面，请校正局面。'
  return null
}
const digits = '一二三四五六七八九'
export function notation(board: Board, move: string): string {
  if(!/^[a-i][0-9][a-i][0-9]$/.test(move)) return move
  const [r,c]=coords(move.slice(0,2)),[y,x]=coords(move.slice(2)),p=board[r][c]
  if(!p) return move
  const red=sideOf(p)==='w', file=(col:number)=>red?digits[8-col]:String(col+1)
  const peers=board.map((row,i)=>row[c]===p?i:-1).filter(i=>i>=0).sort((a,b)=>red?a-b:b-a)
  const prefix=peers.length===2 ? (peers[0]===r?'前':'后')+names[p] : peers.length>2 ? `${peers.length===3&&peers.indexOf(r)===1?'中':digits[peers.indexOf(r)]}${names[p]}${file(c)}` : names[p]+file(c)
  if(y===r) return prefix+'平'+file(x)
  const action=(red?y<r:y>r)?'进':'退'
  const end=['n','b','a'].includes(p.toLowerCase())?file(x):(red?digits[Math.abs(y-r)-1]:String(Math.abs(y-r)))
  return prefix+action+end
}
export function pvNotations(position: Position, pv: string[]): string[] {
  const lines:string[]=[];let pos={board:clone(position.board),side:position.side}
  for(const move of pv.slice(0,12)) { if(!legalMove(pos.board,pos.side,move)) break;lines.push(notation(pos.board,move));pos=applyMove(pos,move) } return lines
}
export function explainMove(board: Board, move: string): string {
  if(!/^[a-i][0-9][a-i][0-9]$/.test(move)) return '当前局面没有可用的推荐走法。'
  const [r,c]=coords(move.slice(0,2)),[y,x]=coords(move.slice(2)),p=board[r][c],target=board[y][x]
  if(!p) return ''
  const next=clone(board);next[y][x]=p;next[r][c]=''
  if(inCheck(next,opposite(sideOf(p)))) return '这步形成将军，迫使对方优先应将。继续查看主变化，留意对方的防守路线。'
  if(target) return `这步吃掉对方的${names[target]}。结合后续变化检查是否存在反吃，避免只看眼前得子。`
  const tips:Record<string,string>={c:'调整炮的位置，关注炮架和中路的配合。',n:'调动马的站位，留意马腿与后续的进攻支点。',r:'调动车的线路，争取开放线和横向活动空间。',p:'推进兵卒，争取空间；过河后注意与大子配合。',a:'调整士仕的防守位置，留意九宫安全。',b:'调整象相的防守，关注象眼和中路。',k:'调整将帅的位置，避开对方的攻击线路。'}
  return tips[p.toLowerCase()]+'这是按着法类型生成的学习提示，具体得失请以引擎主变化为准。'
}
