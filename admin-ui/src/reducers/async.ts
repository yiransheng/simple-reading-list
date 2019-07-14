import {StrictReducer, createStateMachineReducer} from '../utils';
import {SyncAction, Action} from '../actions';

export interface RequestSet {
  has(reqToken: string, reqId: number): boolean;

  isEmpty(): boolean;
  add(reqToken: string, reqId: number): RequestSet;
  delete(reqToken: string, reqId: number): RequestSet;
}

export type AsyncState<S> = IdleState<S> | PendingState<S>;

export function withAsyncState<S>(
  reducer: StrictReducer<S, SyncAction>,
): StrictReducer<AsyncState<S>, Action> {
  return createStateMachineReducer({
    idle: (state: S, action): AsyncState<S> => {
      switch (action.type) {
        case 'REQUEST': {
          const {requestToken, requestId, blocking} = action.payload;
          const requests = new (blocking ? BlockingSet : NonBlockingSet)(
            requestToken,
            requestId,
          );
          return {
            tag: 'pending',
            value: {
              requests,
              state,
            },
          };
        }
        case 'RESPONSE': {
          return {tag: 'idle', value: state};
        }
        default:
          return {tag: 'idle', value: reducer(state, action)};
      }
    },
    pending: (st, action): AsyncState<S> => {
      const {requests, state} = st;
      switch (action.type) {
        case 'REQUEST': {
          const {requestToken, requestId} = action.payload;
          return {
            tag: 'pending',
            value: {
              requests: requests.add(requestToken, requestId),
              state,
            },
          };
        }
        case 'RESPONSE': {
          const {action: innerAction, requestToken, requestId} = action.payload;
          if (!requests.has(requestToken, requestId)) {
            return {tag: 'pending', value: st};
          } else {
            const nextState = reducer(state, innerAction);
            const reqs = requests.delete(requestToken, requestId);
            if (reqs.isEmpty()) {
              return {tag: 'idle', value: nextState};
            } else {
              return {
                tag: 'pending',
                value: {requests: reqs, state: nextState},
              };
            }
          }
        }
        default:
          return {tag: 'pending', value: st};
      }
    },
  });
}

interface IdleState<S> {
  tag: 'idle';
  value: S;
}
interface PendingState<S> {
  tag: 'pending';
  value: {
    requests: RequestSet;
    state: S;
  };
}

class BlockingSet implements RequestSet {
  constructor(
    private reqToken: string | undefined,
    private reqId: number | undefined,
  ) {}

  isEmpty() {
    return !this.reqToken && !this.reqId;
  }
  has(reqToken: string, reqId: number): boolean {
    return this.reqToken === reqToken && this.reqId === reqId;
  }
  add(reqToken: string, reqId: number): BlockingSet {
    return this;
  }
  delete(reqToken: string, reqId: number): BlockingSet {
    if (this.has(reqToken, reqId)) {
      this.reqToken = undefined;
      this.reqId = undefined;
    }
    return this;
  }
}

class NonBlockingSet implements RequestSet {
  private readonly requests: Map<string, number> = new Map();

  constructor(reqToken: string, reqId: number) {
    this.requests.set(reqToken, reqId);
  }

  isEmpty() {
    return this.requests.size === 0;
  }
  has(reqToken: string, reqId: number): boolean {
    return this.requests.get(reqToken) === reqId;
  }
  add(reqToken: string, reqId: number): NonBlockingSet {
    this.requests.set(reqToken, reqId);
    return this;
  }
  delete(reqToken: string, reqId: number): NonBlockingSet {
    if (this.has(reqToken, reqId)) {
      this.requests.delete(reqToken);
    }
    return this;
  }
}
