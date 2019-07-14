import {Reducer} from 'redux';
import {State as SyncState} from '../state';
import {Action, SyncAction} from '../actions';
import {workflowReducer} from './workflow';
import {withAsyncState, AsyncState} from './async';
import {withInitialState, StrictReducer} from '../utils';

function compose<T, U, V>(f: (x: U) => V, g: (y: T) => U): (x: T) => V {
  return x => f(g(x));
}

export type State = AsyncState<SyncState>;

export const rootReducer: Reducer<State, Action> = compose<
  StrictReducer<SyncState, SyncAction>,
  StrictReducer<State, Action>,
  Reducer<State, Action>
>(
  withInitialState<State, Action>({
    tag: 'idle',
    value: {tag: 'unknown', value: null},
  }),
  withAsyncState,
)(workflowReducer);
