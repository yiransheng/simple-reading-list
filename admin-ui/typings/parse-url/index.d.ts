declare module 'parse-url' {
  interface Url {
    protocols: string[],
    protocol: string,
    resource: string,
    user: string,
    pathname: string,
    hash: string,
    search: string,
    href: string,
  }

  declare function parseUrl(url: string): Url;

  export = parseUrl;
}

