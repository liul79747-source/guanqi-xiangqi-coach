<script setup lang="ts">
import { computed } from 'vue'
import type { Board } from '../types'
import { names, sideOf, square, coords } from '../chess'
const props=defineProps<{board:Board;flipped:boolean;selected:string;targets:string[];arrow:string;lastMove:string}>()
const emit=defineEmits<{select:[square:string]}>()
const point=(r:number,c:number)=>({x:48+(props.flipped?8-c:c)*58,y:48+(props.flipped?9-r:r)*58})
const cells=computed(()=>props.board.flatMap((row,r)=>row.map((piece,c)=>({piece,r,c,id:square(r,c),...point(r,c)}))))
const arrowPoints=computed(()=>{if(!/^[a-i][0-9][a-i][0-9]$/.test(props.arrow))return null;const [r,c]=coords(props.arrow.slice(0,2)),[y,x]=coords(props.arrow.slice(2));const a=point(r,c),b=point(y,x),len=Math.hypot(b.x-a.x,b.y-a.y);return{x1:a.x,y1:a.y,x2:b.x-(b.x-a.x)*17/len,y2:b.y-(b.y-a.y)*17/len}})
</script>
<template>
  <svg class="chessboard" viewBox="0 0 560 624" role="group" aria-label="中国象棋棋盘，点击棋子后点击目标位置走棋">
    <defs>
      <linearGradient id="wood" x2="1" y2="1"><stop stop-color="#f6e6c8"/><stop offset=".55" stop-color="#eed9b4"/><stop offset="1" stop-color="#e9cfa2"/></linearGradient>
      <radialGradient id="piece" cx=".35" cy=".25"><stop stop-color="#fff7e6"/><stop offset="1" stop-color="#f3dfbb"/></radialGradient>
      <filter id="shadow" x="-30%" y="-30%" width="160%" height="170%"><feDropShadow dx="0" dy="2" stdDeviation="1.7" flood-color="#5e4126" flood-opacity=".26"/></filter>
      <marker id="arrowhead" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="4" markerHeight="4" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#2e7961"/></marker>
    </defs>
    <rect x="1" y="1" width="558" height="622" rx="9" fill="url(#wood)" stroke="#dec49b" stroke-width="2"/>
    <rect x="40" y="40" width="480" height="538" fill="none" stroke="#806747" stroke-width="2"/>
    <g stroke="#8e7652" stroke-width="1.15" fill="none">
      <path v-for="r in 10" :key="'h'+r" :d="`M48 ${48+(r-1)*58}H512`"/>
      <path v-for="c in 9" :key="'v'+c" :d="c===1||c===9 ? `M${48+(c-1)*58} 48V570` : `M${48+(c-1)*58} 48V280 M${48+(c-1)*58} 338V570`"/>
      <path d="M222 48L338 164 M338 48L222 164 M222 454L338 570 M338 454L222 570"/>
      <g v-for="[r,c] in [[2,1],[2,7],[7,1],[7,7],[3,0],[3,2],[3,4],[3,6],[3,8],[6,0],[6,2],[6,4],[6,6],[6,8]]" :key="`${r}-${c}`">
        <path v-if="c>0" :d="`M${48+c*58-6} ${48+r*58-15}v9h-9 M${48+c*58-6} ${48+r*58+15}v-9h-9`"/>
        <path v-if="c<8" :d="`M${48+c*58+6} ${48+r*58-15}v9h9 M${48+c*58+6} ${48+r*58+15}v-9h9`"/>
      </g>
    </g>
    <g class="river" fill="#8f7452" font-size="29" text-anchor="middle"><text x="165" y="319">楚 河</text><text x="395" y="319">漢 界</text></g>
    <g fill="#927956" font-size="12" text-anchor="middle"><text v-for="c in 9" :key="c" :x="48+(c-1)*58" y="23">{{flipped?'一二三四五六七八九'[c-1]:c}}</text><text v-for="c in 9" :key="'b'+c" :x="48+(c-1)*58" y="606">{{flipped?9-c+1:'九八七六五四三二一'[c-1]}}</text></g>
    <g v-for="cell in cells" :key="cell.id" :transform="`translate(${cell.x},${cell.y})`" class="board-cell" role="button" tabindex="0" :aria-label="`${cell.id} ${cell.piece?names[cell.piece]:'空位'}`" @click="emit('select',cell.id)" @keydown.enter.prevent="emit('select',cell.id)" @keydown.space.prevent="emit('select',cell.id)">
      <rect x="-28" y="-28" width="56" height="56" fill="transparent"/>
      <rect v-if="lastMove.includes(cell.id) && lastMove" x="-26" y="-26" width="52" height="52" rx="5" fill="#4b81602b"/>
      <g v-if="cell.piece" filter="url(#shadow)">
        <circle r="24" fill="url(#piece)" :stroke="selected===cell.id?'#246e56':'#c0a079'" :stroke-width="selected===cell.id?3:1.3"/>
        <circle r="20.3" fill="none" :stroke="sideOf(cell.piece)==='w'?'#b44936':'#4e554a'" stroke-width=".75" opacity=".65"/>
        <text y="10" text-anchor="middle" :fill="sideOf(cell.piece)==='w'?'#ac3a2c':'#303d35'" class="piece-text">{{names[cell.piece]}}</text>
      </g>
      <circle v-if="targets.includes(cell.id)" :r="cell.piece?25:7" :fill="cell.piece?'none':'#387b6299'" stroke="#387b62" stroke-width="2"/>
    </g>
    <line v-if="arrowPoints" v-bind="arrowPoints" stroke="#2e7961" stroke-width="5" stroke-linecap="round" marker-end="url(#arrowhead)" opacity=".88" pointer-events="none"/>
  </svg>
</template>
