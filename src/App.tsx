import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Toaster } from "@/components/ui/sonner";
import { MainPage } from "@/pages/MainPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { SetupWizard } from "@/components/SetupWizard";
import { GetCredentialsResponse } from "@/types";
import "./App.css";

type Page = "main" | "settings" | "setup";

function App() {
  const [currentPage, setCurrentPage] = useState<Page | null>(null);
  const [isCheckingCredentials, setIsCheckingCredentials] = useState(true);

  useEffect(() => {
    const checkFirstLaunch = async () => {
      try {
        const response = await invoke<GetCredentialsResponse>("get_credentials");

        if (!response.has_credentials) {
          // No credentials found, show setup wizard
          setCurrentPage("setup");
        } else {
          // Credentials exist, go to main page
          setCurrentPage("main");
        }
      } catch (error) {
        console.error("Failed to check credentials:", error);
        // On error, show main page
        setCurrentPage("main");
      } finally {
        setIsCheckingCredentials(false);
      }
    };

    checkFirstLaunch();
  }, []);

  const handleSetupComplete = () => {
    setCurrentPage("main");
  };

  const handleSetupSkip = () => {
    setCurrentPage("main");
  };

  if (isCheckingCredentials) {
    // Show loading state while checking credentials
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  return (
    <>
      {currentPage === "main" && (
        <MainPage onSettingsClick={() => setCurrentPage("settings")} />
      )}
      {currentPage === "settings" && (
        <SettingsPage
          onBack={() => setCurrentPage("main")}
          onSetupWizard={() => setCurrentPage("setup")}
        />
      )}
      {currentPage === "setup" && (
        <SetupWizard
          onComplete={handleSetupComplete}
          onSkip={handleSetupSkip}
        />
      )}
      <Toaster />
    </>
  );
}

export default App;
