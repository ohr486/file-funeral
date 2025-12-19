import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";

// Create mocks using vi.hoisted to ensure they're created before module loading
const { mockInvoke, mockOpen } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
  mockOpen: vi.fn(),
}));

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

// Mock @tauri-apps/plugin-dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: mockOpen,
}));

// Mock sonner toast
vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
  Toaster: () => null,
}));

describe("App", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockOpen.mockReset();
    // Set default mock responses
    mockInvoke.mockResolvedValue({
      comparisons: [],
      in_sync_count: 0,
      needs_upload_count: 0,
      needs_download_count: 0,
      conflict_count: 0,
    });
    mockOpen.mockResolvedValue(null);
  });

  describe("Initial Rendering", () => {
    it("should render MainPage by default", () => {
      render(<App />);

      expect(screen.getByText("file-funeral")).toBeInTheDocument();
      expect(screen.getByText("Cloud file synchronization application")).toBeInTheDocument();
    });

    it("should show Settings button on MainPage", () => {
      render(<App />);

      expect(screen.getByRole("button", { name: /Settings/i })).toBeInTheDocument();
    });
  });

  describe("Navigation to Settings", () => {
    it("should navigate to SettingsPage when Settings button is clicked", async () => {
      const user = userEvent.setup();
      render(<App />);

      const settingsButton = screen.getByRole("button", { name: /Settings/i });
      await user.click(settingsButton);

      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
        expect(screen.getByText("Sync Folder")).toBeInTheDocument();
      });
    });

    it("should not show MainPage content when on SettingsPage", async () => {
      const user = userEvent.setup();
      render(<App />);

      const settingsButton = screen.getByRole("button", { name: /Settings/i });
      await user.click(settingsButton);

      await waitFor(() => {
        expect(
          screen.queryByText("Cloud file synchronization application")
        ).not.toBeInTheDocument();
      });
    });

    it("should show Back to Main button on SettingsPage", async () => {
      const user = userEvent.setup();
      render(<App />);

      const settingsButton = screen.getByRole("button", { name: /Settings/i });
      await user.click(settingsButton);

      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Back to Main/i })).toBeInTheDocument();
      });
    });
  });

  describe("Navigation back to Main", () => {
    it("should navigate back to MainPage when Back button is clicked", async () => {
      const user = userEvent.setup();
      render(<App />);

      // Navigate to Settings
      const settingsButton = screen.getByRole("button", { name: /Settings/i });
      await user.click(settingsButton);

      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      // Navigate back to Main
      const backButton = screen.getByRole("button", { name: /Back to Main/i });
      await user.click(backButton);

      await waitFor(() => {
        expect(screen.getByText("Cloud file synchronization application")).toBeInTheDocument();
      });
    });

    it("should not show SettingsPage content when back on MainPage", async () => {
      const user = userEvent.setup();
      render(<App />);

      // Navigate to Settings and back
      const settingsButton = screen.getByRole("button", { name: /Settings/i });
      await user.click(settingsButton);

      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      const backButton = screen.getByRole("button", { name: /Back to Main/i });
      await user.click(backButton);

      await waitFor(() => {
        expect(screen.queryByText("AWS Credentials")).not.toBeInTheDocument();
      });
    });

    it("should show Settings button again when back on MainPage", async () => {
      const user = userEvent.setup();
      render(<App />);

      // Navigate to Settings and back
      const settingsButton = screen.getByRole("button", { name: /Settings/i });
      await user.click(settingsButton);

      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      const backButton = screen.getByRole("button", { name: /Back to Main/i });
      await user.click(backButton);

      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Settings/i })).toBeInTheDocument();
      });
    });
  });

  describe("Multiple Navigation Cycles", () => {
    it("should handle multiple navigation cycles correctly", async () => {
      const user = userEvent.setup();
      render(<App />);

      // First cycle: Main -> Settings -> Main
      await user.click(screen.getByRole("button", { name: /Settings/i }));
      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      await user.click(screen.getByRole("button", { name: /Back to Main/i }));
      await waitFor(() => {
        expect(screen.getByText("Cloud file synchronization application")).toBeInTheDocument();
      });

      // Second cycle: Main -> Settings -> Main
      await user.click(screen.getByRole("button", { name: /Settings/i }));
      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      await user.click(screen.getByRole("button", { name: /Back to Main/i }));
      await waitFor(() => {
        expect(screen.getByText("Cloud file synchronization application")).toBeInTheDocument();
      });
    });
  });

  describe("Toaster", () => {
    it("should render Toaster component", () => {
      const { container } = render(<App />);
      // Toaster is mocked to return null, so we just verify the component renders without errors
      expect(container).toBeInTheDocument();
    });
  });
});
