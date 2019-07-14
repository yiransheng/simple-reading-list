export type PagingState = Idle | Fetching | Failed

export interface Idle {
  tag: 'idle'
  nextPage: number | null
}

export interface Fetching {
  tag: 'fetching'
}

export interface Failed {
  tag: 'failed'
}

export interface Fetched {
  tag: 'fetched'
  data: any
  nextPage: number | null
}

export type Event = PageLoad | RequestNextPage | Loaded | LoadError

interface PageLoad {
  type: 'page_load'
  nextPage: number | null
}

interface RequestNextPage {
  type: 'request_next_page'
}

interface Loaded {
  type: 'loaded'
  nextPage: number | null
  data: any
}

interface LoadError {
  type: 'load_error'
}
