import {State, EMPTY_BOOKMARK} from '../state';
import {SyncAction} from '../actions';
import {createStateMachineReducer, StrictReducer} from '../utils';

export const workflowReducer: StrictReducer<
  State,
  SyncAction
> = createStateMachineReducer({
  unknown(state, action) {
    switch (action.type) {
      case 'LOGIN_ERROR': {
        return {
          tag: 'annoymous',
          value: null
        };
      }
      case 'LOGIN_SUCCESS': {
        return {
          tag: 'admin',
          value: {bookmark: EMPTY_BOOKMARK},
        };
      }
      default:
        return {tag: 'unknown', value: state};
    }
  },
  annoymous(state, action) {
    switch (action.type) {
      case 'LOGIN_SUCCESS': {
        return {
          tag: 'admin',
          value: {bookmark: EMPTY_BOOKMARK},
        };
      }
      default:
        return {tag: 'annoymous', value: state};
    }
  },
  admin(state, action) {
    switch (action.type) {
      case 'LOGOUT': {
        return {
          tag: 'annoymous',
          value: null
        };
      }
      default:
        return {tag: 'admin', value: state};
    }
  },
});
