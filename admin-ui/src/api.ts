import {
  AuthData,
  AdminUser,
  Result,
  Ok,
  Err,
  AuthSuccess,
  GenericError,
  Bookmark,
} from './interface';
import {match} from './utils';

const apiRoot = 'http://localhost:8080/api';

export interface ApiCall<T> {
  (): Promise<T>;
}

export function signin(
  data: AuthData,
): ApiCall<Result<AuthSuccess, GenericError>> {
  return () =>
    fetch(`${apiRoot}/auth`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })
      .then(assertStatusOk)
      .then(res => {
        setToken(res.token);
        return Ok(res);
      })
      // force type casting, needs manual verification
      .catch(error => Err({error})) as any;
}

export const whoami: ApiCall<Result<AuthSuccess, GenericError>> = () =>
  match(getToken(), {
    Ok: (token: string) =>
      fetch(`${apiRoot}/auth`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      })
        .then(assertStatusOk)
        .then(user => Ok({user, token}))
        // force type casting, needs manual verification
        .catch(error => Err({error})) as any,
    Err: () => Promise.resolve(Err({error: 'no token'})) as any,
  });

export const signout: ApiCall<void> = () => {
  deleteToken();
  return Promise.resolve();
};

export function createBookmark(
  data: Bookmark,
): ApiCall<Result<void, GenericError>> {
  return () =>
    match(getToken(), {
      Ok: (token: string) =>
        fetch(`${apiRoot}/auth`, {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(data),
        })
          .then(assertStatusOk)
          .then(Ok)
          // force type casting, needs manual verification
          .catch(error => Err({error})) as any,
      Err: () => Promise.resolve(Err({error: 'no token'})) as any,
    });
}

function assertStatusOk(res: Response) {
  if (res.ok) {
    return res.json();
  } else {
    return res.json().then(err => Promise.reject(err));
  }
}

function setToken(token: string) {
  localStorage.setItem(tokenKey(), token);
}

function deleteToken() {
  localStorage.removeItem(tokenKey());
}

function getToken(): Result<string, null> {
  let token = localStorage.getItem(tokenKey());
  if (token) {
    return Ok(token);
  } else {
    return Err(null);
  }
}

function tokenKey(): string {
  return `${apiRoot}/token`;
}
