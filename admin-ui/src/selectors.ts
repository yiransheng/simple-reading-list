import {State} from './reducers';
import {State as SyncState, AppStatus, DEFAULT_APP_STATUS} from './state';
import {Variant, Option, Some, None, Bookmark} from './interface';
import {match} from './utils';

export function selectIsLoading(state: State): boolean {
  return match(state, {
    pending: () => true,
    _: () => false,
  });
}

export function selectState(state: State): SyncState {
  return match(state, {
    idle: (state: Variant<State, 'idle'>) => state,
    pending: ({state}: Variant<State, 'pending'>) => state,
  });
}

export function selectBookmark(appState: State): Option<Bookmark> {
  const state = selectState(appState);

  return match(state, {
    admin: ({bookmark}: Variant<SyncState, 'admin'>) => Some(bookmark),
    _: () => None(),
  });
}

export function seleteLatestStatus(appState: State): AppStatus {
  const state = selectState(appState);

  return match(state, {
    admin: ({status}: Variant<SyncState, 'admin'>) => status,
    _: () => DEFAULT_APP_STATUS,
  });
}
