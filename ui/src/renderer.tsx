/**
 * This is the entrypoint for Electron's renderer thread.
 */

import { createRoot } from "react-dom/client";
import React from "react";
import UiMain from "./ui_main";

const root = createRoot(document.getElementById("root"));
root.render(<UiMain inMemory={false} />);
