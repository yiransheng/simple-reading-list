import {State, DEFAULT_APP_STATUS, EMPTY_BOOKMARK} from '../state';
import {SyncAction} from '../actions';
import {
  getErrorStatus,
  createStateMachineReducer,
  StrictReducer,
} from '../utils';

export const workflowReducer: StrictReducer<
  State,
  SyncAction
> = createStateMachineReducer({
  unknown(state, action) {
    switch (action.type) {
      case 'LOGIN_ERROR': {
        return {
          tag: 'annoymous',
          value: null,
        };
      }
      case 'LOGIN_SUCCESS': {
        const {user} = action.payload;
        return {
          tag: 'admin',
          value: {bookmark: EMPTY_BOOKMARK, user, status: DEFAULT_APP_STATUS},
        };
      }
      default:
        return {tag: 'unknown', value: state};
    }
  },
  annoymous(state, action) {
    switch (action.type) {
      case 'LOGIN_SUCCESS': {
        const {user} = action.payload;
        return {
          tag: 'admin',
          value: {bookmark: EMPTY_BOOKMARK, user, status: DEFAULT_APP_STATUS},
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
          value: null,
        };
      }
      case 'EDIT_BOOKMARK': {
        return {
          tag: 'admin',
          value: {
            ...state,
            bookmark: action.payload,
          },
        };
      }
      case 'BOOKMARK_CREATED': {
        const {timestamp} = action.payload;
        // TODO: config 2500
        const status = {
          tag: 'ok' as 'ok',
          value: {
            message: 'Created',
            dismissWhen: new Date(timestamp.getTime() + 2500),
          },
        };
        return {
          tag: 'admin',
          value: {
            user: state.user,
            bookmark: EMPTY_BOOKMARK,
            status,
          },
        };
      }
      case 'BOOKMARK_CREATE_FAILURE': {
        const {timestamp} = action.payload;
        const status = getErrorStatus(action.payload, timestamp);
        return {
          tag: 'admin',
          value: {
            user: state.user,
            bookmark: state.bookmark,
            status,
          },
        };
      }
      default:
        return {tag: 'admin', value: state};
    }
  },
});
