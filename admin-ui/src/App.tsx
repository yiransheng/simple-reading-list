import React from "react";
import { connect } from "react-redux";
import { bindActionCreators } from "redux";

import { match } from "./utils";
import { selectState, selectIsLoading } from "./selectors";
import { State } from "./reducers";
import { State as SyncState } from "./state";
import { Variant, Callback, AuthData } from "./interface";
import { AuthForm } from "./components/AuthForm";
import { BookmarkForm } from "./components/Bookmark";
import { LoadingIndicator } from "./components/LoadingIndicator";
import { signin, signout } from "./action_creators";

import "./styles/css/layout.css";

interface Props {
  state: SyncState;

  isLoading: boolean;

  handleSignIn: Callback<AuthData>;

  handleSignOut: Callback<unknown>;
}

const withStoreState = connect(
  (state: State) => ({
    state: selectState(state),
    isLoading: selectIsLoading(state)
  }),
  dispatch =>
    bindActionCreators(
      {
        handleSignIn: signin,
        handleSignOut: signout
      },
      dispatch
    )
);

const App = withStoreState(
  ({ state, isLoading, handleSignIn, handleSignOut }: Props) => {
    return (
      <div className="container">
        <LoadingIndicator show={isLoading} msg="sending request..." />
        {match(state, {
          unknown: () => null,
          annoymous: () => (
            <>
              <h1>Admin Login</h1>
              <div style={{ width: "24rem" }}>
                <AuthForm onSubmit={handleSignIn} />
              </div>
            </>
          ),
          admin: ({ bookmark, user }: Variant<SyncState, "admin">) => (
            <>
              <header>
                <h1>Hello, {user.email}</h1>
                <button onClick={handleSignOut}>Logout</button>
              </header>
              <BookmarkForm
                bookmark={bookmark}
                onSubmit={() => {}}
                onUpdate={() => {}}
              />
            </>
          )
        })}
      </div>
    );
  }
);

export default App;
