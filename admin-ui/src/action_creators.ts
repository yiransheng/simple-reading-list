import { FreeDSL, dispatch, effect, rollback, Do } from "redux-free-flow";

import { Result, AuthData, AuthSuccess, AuthError } from "./interface";
import { signin as signinApi } from "./api";
import { uid, match } from "./utils";

export function signin(data: AuthData): FreeDSL<void> {
  const requestId = uid();
  const requestToken = "signin";

  return Do(function*() {
    yield dispatch({
      type: "REQUEST",
      payload: {
        requestToken,
        requestId
      }
    });
    const result: Result<AuthSuccess, AuthError> = yield effect(
      signinApi(data)
    );

    yield match(result, {
      Ok(res: AuthSuccess) {
        return dispatch({
          type: "RESPONSE",
          payload: {
            requestToken,
            requestId,
            action: { type: "LOGIN_SUCCESS", payload: res }
          }
        });
      },
      _: () => rollback
    });
  });
}
