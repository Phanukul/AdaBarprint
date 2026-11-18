import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Toaster } from "sonner";
import "./globals.css";
import AdaBarprintPage from "./pages/AdaBarprintPage";

if (window.location.pathname === "/") {
  window.history.replaceState(null, "", "/AdaBarprint");
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AdaBarprintPage />
    <Toaster richColors position="top-center" />
  </StrictMode>
);
