import React from 'react';

import '../styles/css/loading_indicator.css';

export interface Props {
  show: boolean;
  msg: string;
  cycleLength?: number;
}

const chars: string[] = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!@#$%^&*()-_=+{}|[]\\;\':"<>?,./`~'.split(
  '',
);

function randomChar(): string {
  const i = Math.floor(Math.random() * chars.length);
  return chars[i];
}

interface AnimationState {
  letters: string[];
  cycleLength: number;
  frame: number;
  on: boolean;
}

function LoadingIndicatorInner(props: AnimationState): JSX.Element {
  const {letters, cycleLength, frame, on} = props;
  const n = letters.length * 2;

  const charIndex = Math.floor(frame / cycleLength) % n;

  const contents = letters.map((char, index) => {
    if (char === ' ') {
      return (
        <span key={index} className="letter">
          &nbsp;
        </span>
      );
    } else if (index <= charIndex) {
      return (
        <span key={index} className="letter">
          {char}
        </span>
      );
    } else {
      return (
        <span key={index} className="glitch" style={{opacity: Math.random()}}>
          {randomChar()}
        </span>
      );
    }
  });
  return (
    <div className="loading-word" style={{opacity: on ? 1.0 : 0.0}}>
      {contents}
    </div>
  );
}

export const LoadingIndicator: React.FC<Props> = props => {
  const {show, msg, cycleLength = 6} = props;
  const letters = msg.split('');
  const frame = useAnimationFrame(
    show,
    cycleLength,
    letters.length * cycleLength,
  );

  return (
    <LoadingIndicatorInner
      letters={letters}
      cycleLength={cycleLength}
      frame={frame}
      on={show}
    />
  );
};

const MULTIPLE = 16;

function useAnimationFrame(on: boolean, start: number, N: number) {
  N = N * MULTIPLE;

  const [frame, setFrame] = React.useState(0);

  React.useEffect(() => {
    if (!on) {
      return () => {};
    }
    let animationFrame: number;
    let stopped = false;

    // Function to be executed on each animation frame

    function eachFrame() {
      setFrame(frame => (frame + 1) % N);
      if (!stopped) {
        loop();
      }
    }

    function loop() {
      animationFrame = requestAnimationFrame(eachFrame);
    }

    setFrame(start);
    loop();

    // Clean things up

    return () => {
      setTimeout(() => {
        cancelAnimationFrame(animationFrame);
        stopped = true;
      }, 500);
    };
  }, [on, start, N]);

  return frame;
}
