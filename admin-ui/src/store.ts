import {createStore} from 'redux';
import {enhancer, EnhancedStore} from 'redux-free-flow';

import {rootReducer, State} from './reducers';
import {Action} from './actions';

export function configureStore(): EnhancedStore<State, Action> {
  return createStore(rootReducer, undefined, enhancer);
}
