// unsubscribe does not guarantee calling complete
interface Unsubscribe {
  (): void
}
interface Subscriber<T> {
  next(value: T): void

  complete(): void
}

type Listener<T> = (x: T) => void

// only models next and complete
export abstract class Observable<T> {
  static of<U>(...values: U[]): Observable<U> {
    return new ObservableOf(values)
  }
  static liftPromise<X, Y>(fn: (x: X) => Promise<Y>): (x: X) => Observable<Y> {
    return x =>
      new FromPromise(() => {
        return fn(x)
      })
  }
  static fromPromiseFactory<U>(f: () => Promise<U>): Observable<U> {
    return new FromPromise(f)
  }
  static fromEventPattern<U>(addListener: (f: Listener<U>) => Unsubscribe): Observable<U> {
    return new FromEvent(addListener)
  }

  subscribe(sub: Subscriber<T>): Unsubscribe {
    throw new Error('Unimplemented')
  }

  map<U>(mapper: (value: T) => U): Observable<U> {
    return new ObservableMap(this, mapper)
  }
  switchMap<U>(mapper: (value: T) => Observable<U>): Observable<U> {
    return new SwitchMap(this, mapper)
  }
}

class ObservableOf<T> extends Observable<T> {
  constructor(private values: Array<T>) {
    super()
  }
  // Override
  subscribe(sub: Subscriber<T>): Unsubscribe {
    for (const value of this.values) {
      sub.next(value)
    }
    sub.complete()

    return () => {}
  }
}

class FromEvent<T> extends Observable<T> {
  constructor(private addListener: (f: Listener<T>) => Unsubscribe) {
    super()
  }
  // Override
  subscribe(sub: Subscriber<T>): Unsubscribe {
    return this.addListener(value => {
      sub.next(value)
    })
  }
}

class FromPromise<T> extends Observable<T> {
  private subscribers: Array<Subscriber<T>> = []

  constructor(private factory: () => Promise<T>) {
    super()
  }
  // Override
  subscribe(sub: Subscriber<T>): Unsubscribe {
    this.subscribers.push(sub)
    // first
    if (this.subscribers.length === 1) {
      this.factory().then(value => {
        const subs = [...this.subscribers]
        for (const sub of subs) {
          sub.next(value)
          sub.complete()
        }
      })
    }

    return () => {
      this.subscribers = this.subscribers.filter(s => s !== sub)
    }
  }
}

class ObservableMap<T, U> extends Observable<U> {
  constructor(private source: Observable<T>, private mapper: (x: T) => U) {
    super()
  }
  // Override
  subscribe(sub: Subscriber<U>): Unsubscribe {
    return this.source.subscribe({
      next: value => sub.next(this.mapper(value)),
      complete: () => sub.complete()
    })
  }
}

class SwitchMap<T, U> extends Observable<U> {
  private unsubSource: Map<Subscriber<U>, Unsubscribe> = new Map()
  private unsubLatest: Map<Subscriber<U>, Unsubscribe> = new Map()

  constructor(private source: Observable<T>, private mapper: (s: T) => Observable<U>) {
    super()
  }

  // Override
  subscribe(sub: Subscriber<U>): Unsubscribe {
    const unsubSource = this.source.subscribe({
      next: value => {
        const unsubLatest = this.unsubLatest.get(sub)
        if (unsubLatest) {
          unsubLatest()
          this.unsubLatest.delete(sub)
        }
        this.unsubLatest.set(
          sub,
          this.mapper(value).subscribe({
            next: value => sub.next(value),
            complete: () => {
              const unsubSource = this.unsubSource.get(sub)
              if (unsubSource) {
                unsubSource()
                this.unsubSource.delete(sub)
              }
              sub.complete()
            }
          })
        )
      },
      complete: () => {
        for (const [sub, unsubSource] of this.unsubSource) {
          if (!this.unsubLatest.has(sub)) {
            unsubSource()
          }
        }
      }
    })

    this.unsubSource.set(sub, unsubSource)

    return () => {
      const unsubSource = this.unsubSource.get(sub)
      if (unsubSource) {
        unsubSource()
        this.unsubSource.delete(sub)
      }
      const unsubLatest = this.unsubLatest.get(sub)
      if (unsubLatest) {
        unsubLatest()
        this.unsubLatest.delete(sub)
      }
    }
  }
}
