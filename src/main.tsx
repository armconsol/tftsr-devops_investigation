// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import App from "./App";
import "./styles/globals.css";

// Use the locally bundled Monaco instead of loading from CDN.
// Tauri's WebView has no internet access so the default CDN loader never resolves.
loader.config({ monaco });

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);
