import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach, beforeEach, vi } from 'vitest'
import { webcrypto } from 'node:crypto'

afterEach(() => cleanup())
beforeEach(()=>{window.history.replaceState(null,'','#/');localStorage.clear()})
window.scrollTo=vi.fn()
window.matchMedia=vi.fn().mockImplementation(query=>({matches:false,media:query,onchange:null,addListener:vi.fn(),removeListener:vi.fn(),addEventListener:vi.fn(),removeEventListener:vi.fn(),dispatchEvent:vi.fn()}))
Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true})
class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}
vi.stubGlobal('ResizeObserver', ResizeObserverMock)
if (!window.PointerEvent) {
  class PointerEventMock extends MouseEvent {
    pointerId = 1
    pointerType = 'mouse'
    isPrimary = true
  }
  vi.stubGlobal('PointerEvent', PointerEventMock)
}
Element.prototype.scrollIntoView = vi.fn()
Element.prototype.hasPointerCapture = vi.fn(() => false)
Element.prototype.setPointerCapture = vi.fn()
Element.prototype.releasePointerCapture = vi.fn()
