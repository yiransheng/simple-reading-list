import {createStore} from 'redux';
import {enhancer, FreeDSL, EnhancedStore} from 'redux-free-flow';

import {rootReducer, State} from './reducers';
import {Action} from './actions';
import {checkToken} from './action_creators';

export type ReduxDSL<T> = FreeDSL<State, Action, T>;

export function configureStore(): EnhancedStore<State, Action> {
  const store = createStore(rootReducer, undefined, enhancer);
  if (process.env.NODE_ENV === 'development') {
    (window as any).store = store;
  }

  store.dispatch(checkToken());

  return store;
}
