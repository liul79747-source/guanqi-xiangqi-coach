import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import App from '../src/App.vue'
import { START_FEN } from '../src/chess'
const mocks=vi.hoisted(()=>({analyze:vi.fn()}))
vi.mock('../src/api',()=>({__v_isRef:false,desktop:false,resources:async()=>({engine:true,model:true,runtime:true,desktop:false,resourceDir:'test-resources'}),analyze:mocks.analyze,listWindows:async()=>[],capture:vi.fn(),startWatch:vi.fn(),stopWatch:vi.fn()}))
let wrapper:VueWrapper
function button(text:string){const found=wrapper.findAll('button').find(b=>b.text().includes(text));if(!found)throw new Error(`Missing button ${text}`);return found}
beforeEach(async()=>{localStorage.clear();mocks.analyze.mockReset();mocks.analyze.mockImplementation(async(fen:string)=>({fen,bestmove:'b2e2',pv:['b2e2','b9c7'],score:31,mate:null,depth:8,time:100,source:'测试引擎'}));wrapper=mount(App,{attachTo:document.body});await flushPromises()})
afterEach(()=>{wrapper?.unmount();document.body.innerHTML='';vi.restoreAllMocks()})
describe('观棋界面交互（内存 DOM，无浏览器）',()=>{
  it('渲染 90 个位置，并能真实移动棋子、保存和悔棋',async()=>{
    expect(wrapper.findAll('.board-cell')).toHaveLength(90)
    await wrapper.get('[aria-label="b2 炮"]').trigger('click')
    await wrapper.get('[aria-label="e2 空位"]').trigger('click')
    expect(wrapper.find('.turn-pill').text()).toContain('黑方')
    expect(wrapper.find('[aria-label="e2 炮"]').exists()).toBe(true)
    expect(JSON.parse(localStorage.getItem('guanqi-session')!).history.at(-1).move).toBe('b2e2')
    await button('悔棋').trigger('click')
    expect(wrapper.find('[aria-label="b2 炮"]').exists()).toBe(true)
    expect(wrapper.find('.turn-pill').text()).toContain('红方')
  })
  it('分析展示推荐、评分和主变化，走出推荐后不保留旧评分',async()=>{
    await button('分析局面').trigger('click');await flushPromises()
    expect(mocks.analyze).toHaveBeenCalledWith(START_FEN,expect.objectContaining({depth:20}))
    expect(wrapper.find('.move-row').text()).toContain('炮八平五')
    expect(wrapper.find('.score-row').text()).toContain('+0.31')
    expect(wrapper.findAll('.pv-list>span')).toHaveLength(2)
    await button('走出此步').trigger('click')
    expect(wrapper.find('.turn-pill').text()).toContain('黑方')
    expect(wrapper.find('.score-row').text()).not.toContain('+0.31')
  })
  it('在计算中改棋，旧结果不会覆盖新局面',async()=>{
    let resolve!:(v:unknown)=>void;mocks.analyze.mockImplementation(()=>new Promise(r=>resolve=r))
    await button('分析局面').trigger('click')
    await wrapper.get('[aria-label="b2 炮"]').trigger('click');await wrapper.get('[aria-label="e2 空位"]').trigger('click')
    resolve({fen:START_FEN,bestmove:'b2e2',pv:['b2e2'],score:100,mate:null,depth:8,time:100,source:'测试引擎'});await flushPromises()
    expect(wrapper.find('.move-row').text()).toContain('等待分析')
    expect(wrapper.find('.turn-pill').text()).toContain('黑方')
  })
  it('自由摆棋可删除棋子，完成时提示无将帅的非法局面',async()=>{
    await button('摆棋模式').trigger('click');await wrapper.get('[aria-label="移除棋子"]').trigger('click');await wrapper.get('[aria-label="e0 帥"]').trigger('click');await button('完成摆棋').trigger('click')
    expect(wrapper.find('[role="alert"]').text()).toContain('红帅和黑将')
  })
  it('浏览器模式清楚标明窗口识别需要桌面版',async()=>{
    await button('窗口识别').trigger('click')
    expect(wrapper.find('.capture-panel').text()).toContain('窗口截图需打开 Tauri 桌面版')
    expect(button('选择窗口').attributes('disabled')).toBeDefined()
  })
})
