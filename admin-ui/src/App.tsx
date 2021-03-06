import React from 'react';
import {connect} from 'react-redux';
import {bindActionCreators} from 'redux';

import {match} from './utils';
import {selectState, selectLatestStatus, selectIsLoading} from './selectors';
import {State} from './reducers';
import {State as SyncState, AppStatus} from './state';
import {Variant, Callback, AuthData, Bookmark} from './interface';
import {AuthForm} from './components/AuthForm';
import {BookmarkForm} from './components/Bookmark';
import {StatusBar} from './components/StatusBar';
import {LoadingIndicator} from './components/LoadingIndicator';
import {signin, signout, editBookmark, createBookmark} from './action_creators';

import './styles/css/layout.css';

interface Props {
  state: SyncState;

  isLoading: boolean;

  latestStatus: AppStatus;

  handleSignIn: Callback<AuthData>;

  handleSignOut: Callback<unknown>;

  handleEdit: Callback<Bookmark>;

  handleSubmit: Callback<unknown>;
}

const withStoreState = connect(
  (state: State) => ({
    state: selectState(state),
    isLoading: selectIsLoading(state),
    latestStatus: selectLatestStatus(state),
  }),
  dispatch =>
    bindActionCreators(
      {
        handleSignIn: signin,
        handleSignOut: signout,
        handleEdit: editBookmark,
        handleSubmit: createBookmark,
      },
      dispatch,
    ),
);

const dead = new Date(Date.now() + 2000);
const App = withStoreState(
  ({
    state,
    isLoading,
    latestStatus,
    handleSignIn,
    handleSignOut,
    handleSubmit,
    handleEdit,
  }: Props) => {
    return (
      <div className="container">
        <LoadingIndicator show={isLoading} msg="sending request..." />
        {match(state, {
          unknown: () => null,
          annoymous: () => (
            <>
              <h1>Admin Login</h1>
              <div style={{width: '24rem'}}>
                <AuthForm onSubmit={handleSignIn} />
              </div>
            </>
          ),
          admin: ({bookmark, user}: Variant<SyncState, 'admin'>) => (
            <>
              <header>
                <h1>Hello, {user.email}</h1>
                <button onClick={handleSignOut}>Logout</button>
              </header>
              <StatusBar status={latestStatus} />
              <BookmarkForm
                bookmark={bookmark}
                onSubmit={handleSubmit}
                onUpdate={handleEdit}
              />
            </>
          ),
        })}
      </div>
    );
  },
);

export default App;
