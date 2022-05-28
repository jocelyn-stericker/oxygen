import React from "react";
import cx from "classnames";

import { Record, Stop } from "./icons";

export default function RecordTab({
  streaming,
  onRecord,
  onStop,
}: {
  streaming: boolean;
  onRecord: () => void;
  onStop: () => void;
}) {
  return (
    <div className="flex flex-row flex-grow">
      <div className="flex-grow" />
      <button
        data-testid="toggle-record"
        className={cx(
          "p-4 rounded-md m-auto text-lg flex border-2",
          streaming
            ? "bg-white border-purple-900 text-purple-900 hover:bg-purple-100"
            : "bg-purple-900 text-white hover:bg-purple-800"
        )}
        onClick={(ev) => {
          ev.preventDefault();
          if (!streaming) {
            onRecord();
          } else {
            onStop();
          }
        }}
      >
        {streaming ? (
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
  );
}
