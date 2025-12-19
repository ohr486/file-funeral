import { useState } from "react";
import { Toaster } from "@/components/ui/sonner";
import { MainPage } from "@/pages/MainPage";
import { SettingsPage } from "@/pages/SettingsPage";
import "./App.css";

type Page = "main" | "settings";

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("main");

  return (
    <>
      {currentPage === "main" && (
        <MainPage onSettingsClick={() => setCurrentPage("settings")} />
      )}
      {currentPage === "settings" && (
        <SettingsPage onBack={() => setCurrentPage("main")} />
      )}
      <Toaster />
    </>
  );
}

export default App;
