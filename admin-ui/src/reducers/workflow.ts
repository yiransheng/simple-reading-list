import { State, EMPTY_BOOKMARK } from "../state";
import { SyncAction } from "../actions";
import { createStateMachineReducer, StrictReducer } from "../utils";

export const workflowReducer: StrictReducer<
  State,
  SyncAction
> = createStateMachineReducer({
  annoymous(state, action) {
    switch (action.type) {
      case "LOGIN_SUCCESS": {
        return {
          tag: "admin",
          value: { bookmark: EMPTY_BOOKMARK }
        };
      }
      default:
        return { tag: "annoymous", value: state };
    }
  },
  admin(state, action) {
    return {
      tag: "admin",
      value: state
    };
  }
});
