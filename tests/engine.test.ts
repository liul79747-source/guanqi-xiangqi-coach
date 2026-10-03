import { test } from 'node:test'
import assert from 'node:assert/strict'
// @ts-ignore -- runnable Node adapter shared by the browser edition
import { analyze, parseInfo, validateSettings } from '../scripts/engine.mjs'
import { START_FEN, parseFen, legalMove } from '../src/chess'
test('UCI 多行评分和主变化解析',()=>{const r:any={};parseInfo('info depth 20 score mate 4 pv b2e2 h9g7',r);assert.equal(r.mate,4);parseInfo('info depth 21 score cp -32 pv b2e2 b9c7',r);assert.equal(r.mate,null);assert.equal(r.score,-32);assert.equal(r.pv.length,2)})
test('拒绝非数值和超大引擎配置',()=>{assert.throws(()=>validateSettings({depth:99}));assert.throws(()=>validateSettings({depth:null}))})
test('真实 Pikafish 本机分析返回合法着法',{timeout:30000},async()=>{const r=await analyze(START_FEN,{depth:8,time:500,threads:1,hash:32,cloud:false,cloudTimeout:1});const p=parseFen(START_FEN);assert.equal(r.source,'Pikafish 本机');assert.equal(legalMove(p.board,p.side,r.bestmove),true);assert.ok(r.depth>0);assert.equal(r.pv[0],r.bestmove)})
