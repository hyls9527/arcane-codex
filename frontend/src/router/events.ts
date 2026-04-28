import { listen, emit } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { Page } from '@/types/image'

export type AppRoute = Page

export type NavigationSource = 'sidebar' | 'action' | 'tauri-event' | 'keyboard' | 'system'

export interface RoutePayload {
  route: AppRoute
  params?: Record<string, string>
  source?: NavigationSource
}

export const ROUTE_CHANGE = 'app:route-change'
export const ROUTE_BACK = 'app:route-back'
export const ROUTE_FORWARD = 'app:route-forward'

export const navigate = (payload: RoutePayload) => emit(ROUTE_CHANGE, payload)

export const goBack = () => emit(ROUTE_BACK, {})

export const goForward = () => emit(ROUTE_FORWARD, {})

export function setupTauriRouteListeners(): Promise<UnlistenFn[]> {
  const unsubs: Promise<UnlistenFn>[] = []

  unsubs.push(
    listen('tauri-navigate', (event) => {
      navigate({ ...event.payload as RoutePayload, source: 'tauri-event' })
    })
  )

  return Promise.all(unsubs)
}
