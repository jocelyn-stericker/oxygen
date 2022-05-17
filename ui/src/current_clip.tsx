import { UiState } from "oxygen-core";
import cx from "classnames";
import React from "react";
import { Pause, Play, Delete } from "./icons";

export default function CurrentClip({ uiState }: { uiState: UiState }) {
  const clip = uiState.currentClip;
  return (
    <div className="flex flex-col flex-grow">
      <div className="flex flex-row">
        <div className="self-center p-4 text-m font-bold overflow-ellipses overflow-hidden">
          {clip.name}
        </div>
        <div className="flex-grow" />
        <button
          className="p-2 m-2 text-purple-900 cursor-pointer border-2 border-transparent hover:border-red-900 rounded-full hover:bg-red-100 hover:text-red-900"
          title="Delete this clip"
          onClick={(ev) => {
            uiState.deleteClip();
            ev.preventDefault();
          }}
        >
          <Delete />
        </button>
      </div>
      <div className="flex-grow" />
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
              uiState.play(() => {
                uiState.stop();
              });
            } else {
              uiState.stop();
            }
          }}
        >
          {uiState.streaming ? (
            <>
              <Pause />
              <span className="w-2" />
              Pause
            </>
          ) : (
            <>
              <Play />
              <span className="w-2" />
              Play
            </>
          )}
        </button>
        <div className="flex-grow" />
      </div>
      <div className="flex-grow" />
    </div>
  );
}
