import { UiState } from "oxygen-core";
import { createRoot } from "react-dom/client";
import React, { useState, useCallback } from "react";
import cx from "classnames";
import { Record, Stop } from "./icons";

import CurrentClip from "./current_clip";

function Main() {
  // Hack to force a re-render when the state changes.
  const [, setUpdateSymbol] = useState({});
  const updateCallback = useCallback(() => {
    setUpdateSymbol({});
  }, [setUpdateSymbol]);

  const [uiState] = useState(() => new UiState(updateCallback));
  const clips = uiState.getClips().reverse();
  return (
    <div className="w-screen h-screen flex flex-row">
      <ul className="w-72 border-r-purple-900 border-r-2 h-full divide-y divide-purple-200 overflow-y-auto">
        <li
          className="hover:bg-purple-100 cursor-pointer text-purple-900 overflow-hidden"
          onClick={(ev) => {
            ev.preventDefault();
            uiState.setCurrentTabRecord();
          }}
        >
          <h2
            className={cx(
              "p-4 text-m font-bold overflow-ellipsis overflow-hidden flex flex-row justify-center",
              uiState.recordTabSelected &&
                "bg-purple-900 text-white hover:bg-purple-900 cursor-default"
            )}
          >
            <Record /> Record New Clip
          </h2>
        </li>
        {clips.map((clip) => (
          <li
            key={clip.id.toString()}
            className={cx(
              "p-2 hover:bg-purple-100 cursor-pointer text-purple-900 overflow-hidden",
              uiState.currentClipId === clip.id &&
                "bg-purple-900 text-white hover:bg-purple-900 cursor-default"
            )}
            onClick={(ev) => {
              ev.preventDefault();
              uiState.setCurrentClipId(Number(clip.id));
            }}
          >
            <h2
              className="text-m font-bold overflow-ellipsis overflow-hidden"
              title={clip.name}
            >
              {clip.name}
            </h2>
            <div className="flex flex-row">
              <div className="text-xs font-light">
                {clip.date.toDateString()} at {clip.date.toLocaleTimeString()}
              </div>
            </div>
          </li>
        ))}
      </ul>
      {uiState.currentClipId != null && <CurrentClip uiState={uiState} />}
      {uiState.recordTabSelected && (
        <div className="flex flex-row flex-grow">
          <div className="flex-grow" />
          <button
            className={cx(
              "p-4 rounded-md m-auto text-lg flex border-2",
              uiState.streaming
                ? "bg-white border-purple-900 text-purple-900 hover:bg-purple-100"
                : "bg-purple-900 text-white hover:bg-purple-800"
            )}
            onClick={(ev) => {
              ev.preventDefault();
              if (!uiState.streaming) {
                uiState.record();
              } else {
                uiState.stop();
              }
            }}
          >
            {uiState.streaming ? (
              <>
                <Stop />
                <span className="w-2" />
                Complete Recording
              </>
            ) : (
              <>
                <Record />
                <span className="w-2" />
                Start Recording
              </>
            )}
          </button>
          <div className="flex-grow" />
        </div>
      )}
    </div>
  );
}

const root = createRoot(document.getElementById("root"));
root.render(<Main />);