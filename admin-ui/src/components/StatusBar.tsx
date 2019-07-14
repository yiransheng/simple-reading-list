import React from 'react';

import {AppStatus} from '../state';
import {Variant} from '../interface';
import {match} from '../utils';

import '../styles/css/status_bar.css';

const SCRIPT_LOADED_WHEN = new Date();

export interface Props {
  status: AppStatus;
}

function useAutoDismiss(dismissWhen: Date) {
  const [wakeUp, setWakeUp] = React.useState(new Date());
  const ts = dismissWhen.getTime();

  React.useEffect(() => {
    const now = Date.now();
    const duration = ts - now;
    if (duration < 0) {
      return;
    }

    const timer = setTimeout(() => {
      setWakeUp(() => new Date());
    }, duration + 60);

    return () => {
      clearTimeout(timer);
    };
  }, [ts]);

  return wakeUp;
}

export function StatusBar(props: Props): JSX.Element {
  const {status} = props;
  const dismissWhen = match(status, {
    ok({dismissWhen}: Variant<AppStatus, 'ok'>) {
      return dismissWhen;
    },
    err({dismissWhen}: Variant<AppStatus, 'err'>) {
      return dismissWhen;
    },
    _: () => SCRIPT_LOADED_WHEN,
  });

  const wakeUp = useAutoDismiss(dismissWhen);

  return match(status, {
    ok({message, dismissWhen}: Variant<AppStatus, 'ok'>) {
      if (wakeUp <= dismissWhen) {
        return (
          <div className="status_bar success">
            <span>{message}</span>
          </div>
        );
      } else {
        return NO_STATUS;
      }
    },
    err({message, dismissWhen}: Variant<AppStatus, 'err'>) {
      if (wakeUp <= dismissWhen) {
        return (
          <div className="status_bar error">
            <span>{message}</span>
          </div>
        );
      } else {
        return NO_STATUS;
      }
    },
    _: () => NO_STATUS,
  });
}

const NO_STATUS: JSX.Element = <div className="status_bar none"></div>;
