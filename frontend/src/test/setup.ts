import '@testing-library/jest-dom'
import { cleanup } from '@testing-library/react'
import { afterEach, vi, beforeAll } from 'vitest'

process.env.NODE_ENV = 'test'

if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  })

  class MockResizeObserver {
      callback: ResizeObserverCallback
      constructor(callback: ResizeObserverCallback) {
        this.callback = callback
      }
      observe(element: Element) {
        const mockEntry: Partial<ResizeObserverEntry> = {
          contentRect: { x: 0, y: 0, width: 800, height: 600, top: 0, bottom: 600, left: 0, right: 800, toJSON: () => {} } as DOMRectReadOnly,
          target: element,
        }
        this.callback([mockEntry as ResizeObserverEntry], this)
      }
      unobserve() {}
      disconnect() {}
    }

  if (!window.ResizeObserver) {
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver
  }
}

vi.mock('i18next', () => ({
  createInstance: vi.fn(() => ({
    use: vi.fn().mockReturnValue({ init: vi.fn() }),
    init: vi.fn(),
    changeLanguage: vi.fn(),
    on: vi.fn(),
  })),
}))

afterEach(() => {
  cleanup()
})
