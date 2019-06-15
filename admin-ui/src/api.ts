import { AuthData, Result, Ok, Err, AuthSuccess, AuthError } from "./interface";

const apiRoot = "http://localhost:8080/api";

export interface ApiCall<T> {
  (): Promise<T>;
}

export function signin(
  data: AuthData
): ApiCall<Result<AuthSuccess, AuthError>> {
  return () =>
    fetch(`${apiRoot}/auth`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(data)
    })
      .then(assertStatusOk)
      .then(Ok)
      // force type casting, needs manual verification
      .catch(error => Err({ error })) as any;
}

export function validateToken(
  token: string,
): ApiCall<Result<void, AuthError>> {
  return () =>
    fetch(`${apiRoot}/auth`, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json"
      },
    })
      .then(assertStatusOk)
      .then(Ok)
      // force type casting, needs manual verification
      .catch(error => Err({ error })) as any;
}

function assertStatusOk(res: Response) {
  if (res.ok) {
    return res.json();
  } else {
    return res.json().then(err => Promise.reject(err));
  }
}
