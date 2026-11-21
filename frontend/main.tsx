import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Toaster } from "sonner";
import "./globals.css";
import AdaBarprintPage from "./pages/AdaBarprintPage";
import { installApiSession } from './lib/apiSession';
import { AdminAuthorization } from './components/AdminAuthorization';

installApiSession();

if (window.location.pathname === "/") {
  window.history.replaceState(null, "", "/AdaBarprint");
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AdaBarprintPage />
    <AdminAuthorization />
    <Toaster richColors position="top-center" />
  </StrictMode>
);
