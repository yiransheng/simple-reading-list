import { Observable } from './observable'
import { JsonMl, createElement } from './jsonml'

type Api<Req, Res> = (req: Req) => Observable<Res>

interface MoreItems {
  data: Array<JsonMl>
  next_page: number | undefined
}

const fetchMore: Api<number, MoreItems> = Observable.liftPromise(nextPage => {
  return fetch(`/api/bookmarks/page/${nextPage}`)
    .then(res => {
      if (res.ok) {
        return res.json()
      }
      throw new Error(res.statusText)
    })
    .catch((err: unknown) => ({
      data: [getErrUi(err)]
    }))
})

const clicks$: Observable<number> = Observable.fromEventPattern(listener => {
  function onClick(e: MouseEvent) {
    const { target } = e
    if (target instanceof HTMLElement) {
      const { nextPage: page } = target.dataset
      const nextPage = parseInt(page || 'NaN', 10)
      if (Number.isFinite(nextPage)) {
        e.preventDefault()
        listener(nextPage)
      }
    }
  }

  document.addEventListener('click', onClick)

  return () => document.removeEventListener('click', onClick)
})

const data$ = clicks$.switchMap(fetchMore)

// DOM

function getErrUi(err: unknown): JsonMl {
  let message: string

  if (typeof err === 'string') {
    message = err
  } else if (err instanceof Error && err.message) {
    message = err.message
  } else {
    message = 'Something went wrong'
  }

  return ['div', { class: 'item' }, ['p', ['span', { class: 'error' }, message]]]
}

function getButtonUi(nextPage: number): JsonMl {
  return ['div', { class: 'item' }, ['a', { 'data-next-page': nextPage.toString() }, 'More']]
}

const buttonSelector = '[data-next-page]'
const containerSelector = '.main'

function noop() {}

export function main() {
  const doc = document
  const body = doc.querySelector(containerSelector)

  clicks$.subscribe({
    next: nextPage => {
      const btn = doc.querySelector(buttonSelector)
      if (btn) {
        btn.parentNode && btn.parentNode.removeChild(btn)
      }
    },
    complete: noop
  })

  data$.subscribe({
    next: ({ data, next_page }) => {
      if (!body) {
        return
      }
      for (const d of data) {
        const el = createElement(d)
        body.appendChild(el)
      }
      if (next_page != null && Number.isFinite(next_page)) {
        const btn = createElement(getButtonUi(next_page))
        body.appendChild(btn)
      }
    },
    complete: noop
  })
}
