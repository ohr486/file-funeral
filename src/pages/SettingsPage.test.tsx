import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SettingsPage } from "./SettingsPage";

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
}));

describe("SettingsPage", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockOpen.mockReset();
  });

  describe("Rendering", () => {
    it("should render the settings page with all sections", () => {
      render(<SettingsPage />);

      expect(screen.getByText("Settings")).toBeInTheDocument();
      expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      expect(screen.getByText("Sync Folder")).toBeInTheDocument();
      expect(screen.getByText("Need Help?")).toBeInTheDocument();
    });

    it("should render all form fields", () => {
      render(<SettingsPage />);

      expect(screen.getByLabelText(/Access Key ID/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Secret Access Key/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Region/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/S3 Bucket Name/i)).toBeInTheDocument();
    });

    it("should render action buttons", () => {
      render(<SettingsPage />);

      expect(screen.getByRole("button", { name: /Save Credentials/i })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /Test Connection/i })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /Browse/i })).toBeInTheDocument();
    });

    it("should render back button when onBack prop is provided", () => {
      const mockOnBack = vi.fn();
      render(<SettingsPage onBack={mockOnBack} />);

      expect(screen.getByRole("button", { name: /Back to Main/i })).toBeInTheDocument();
    });

    it("should not render back button when onBack prop is not provided", () => {
      render(<SettingsPage />);

      expect(screen.queryByRole("button", { name: /Back to Main/i })).not.toBeInTheDocument();
    });
  });

  describe("Form Input", () => {
    it("should update access key id field when typed", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const input = screen.getByLabelText(/Access Key ID/i) as HTMLInputElement;
      await user.type(input, "AKIAIOSFODNN7EXAMPLE");

      expect(input.value).toBe("AKIAIOSFODNN7EXAMPLE");
    });

    it("should update secret access key field when typed", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const input = screen.getByLabelText(/Secret Access Key/i) as HTMLInputElement;
      await user.type(input, "secretkey123");

      expect(input.value).toBe("secretkey123");
    });

    it("should update bucket name field when typed", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const input = screen.getByLabelText(/S3 Bucket Name/i) as HTMLInputElement;
      await user.type(input, "my-bucket");

      expect(input.value).toBe("my-bucket");
    });

    it("should update region when selected", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const select = screen.getByLabelText(/Region/i) as HTMLSelectElement;
      await user.selectOptions(select, "us-west-2");

      expect(select.value).toBe("us-west-2");
    });
  });

  describe("Form Validation", () => {
    it("should show validation error when access key id is empty", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText("Access Key ID is required")).toBeInTheDocument();
      });
    });

    it("should show validation error when secret access key is empty", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const accessKeyInput = screen.getByLabelText(/Access Key ID/i);
      await user.type(accessKeyInput, "AKIAIOSFODNN7EXAMPLE");

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText("Secret Access Key is required")).toBeInTheDocument();
      });
    });

    it("should show validation error when bucket name is empty", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const accessKeyInput = screen.getByLabelText(/Access Key ID/i);
      await user.type(accessKeyInput, "AKIAIOSFODNN7EXAMPLE");

      const secretKeyInput = screen.getByLabelText(/Secret Access Key/i);
      await user.type(secretKeyInput, "secretkey123");

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText("Bucket name is required")).toBeInTheDocument();
      });
    });

    it("should show validation error for invalid bucket name format", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const accessKeyInput = screen.getByLabelText(/Access Key ID/i);
      await user.type(accessKeyInput, "AKIAIOSFODNN7EXAMPLE");

      const secretKeyInput = screen.getByLabelText(/Secret Access Key/i);
      await user.type(secretKeyInput, "secretkey123");

      const bucketInput = screen.getByLabelText(/S3 Bucket Name/i);
      await user.type(bucketInput, "Invalid_Bucket_Name");

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText("Invalid bucket name format")).toBeInTheDocument();
      });
    });

    it("should clear validation error when field is corrected", async () => {
      const user = userEvent.setup();
      render(<SettingsPage />);

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(screen.getByText("Access Key ID is required")).toBeInTheDocument();
      });

      const accessKeyInput = screen.getByLabelText(/Access Key ID/i);
      await user.type(accessKeyInput, "AKIAIOSFODNN7EXAMPLE");

      await waitFor(() => {
        expect(screen.queryByText("Access Key ID is required")).not.toBeInTheDocument();
      });
    });
  });

  describe("Save Credentials", () => {
    it("should call set_credentials command with correct data", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({ success: true, message: "Saved" });

      render(<SettingsPage />);

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.selectOptions(screen.getByLabelText(/Region/i), "us-west-2");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("set_credentials", {
          request: {
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret_access_key: "secretkey123",
            region: "us-west-2",
            bucket_name: "my-bucket",
          },
        });
      });
    });

    it("should disable save button while saving", async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({ success: true }), 100))
      );

      render(<SettingsPage />);

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      expect(saveButton).toBeDisabled();

      await waitFor(() => {
        expect(saveButton).not.toBeDisabled();
      });
    });

    it("should clear secret access key after successful save", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({ success: true, message: "Saved" });

      render(<SettingsPage />);

      const secretKeyInput = screen.getByLabelText(/Secret Access Key/i) as HTMLInputElement;

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(secretKeyInput, "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const saveButton = screen.getByRole("button", { name: /Save Credentials/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(secretKeyInput.value).toBe("");
      });
    });
  });

  describe("Test Connection", () => {
    it("should call test_connection command when button is clicked", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({
        connected: true,
        message: "Connected successfully",
        region: "us-east-1",
        bucket_name: "test-bucket",
      });

      render(<SettingsPage />);

      const testButton = screen.getByRole("button", { name: /Test Connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("test_connection");
      });
    });

    it("should display success message when connection succeeds", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({
        connected: true,
        message: "Connected successfully",
        region: "us-east-1",
        bucket_name: "test-bucket",
      });

      render(<SettingsPage />);

      const testButton = screen.getByRole("button", { name: /Test Connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText("Connected successfully")).toBeInTheDocument();
      });
    });

    it("should display error message when connection fails", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({
        connected: false,
        message: "Connection failed",
      });

      render(<SettingsPage />);

      const testButton = screen.getByRole("button", { name: /Test Connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText("Connection failed")).toBeInTheDocument();
      });
    });

    it("should disable test button while testing", async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) =>
            setTimeout(() => resolve({ connected: true, message: "Success" }), 100)
          )
      );

      render(<SettingsPage />);

      const testButton = screen.getByRole("button", { name: /Test Connection/i });
      await user.click(testButton);

      expect(testButton).toBeDisabled();
      expect(testButton).toHaveTextContent("Testing...");

      await waitFor(() => {
        expect(testButton).not.toBeDisabled();
        expect(testButton).toHaveTextContent("Test Connection");
      });
    });
  });

  describe("Folder Selection", () => {
    it("should call open dialog when browse button is clicked", async () => {
      const user = userEvent.setup();
      mockOpen.mockResolvedValue("/path/to/folder");

      render(<SettingsPage />);

      const browseButton = screen.getByRole("button", { name: /Browse/i });
      await user.click(browseButton);

      await waitFor(() => {
        expect(mockOpen).toHaveBeenCalledWith({
          directory: true,
          multiple: false,
          title: "Select folder to sync",
        });
      });
    });

    it("should update folder path when folder is selected", async () => {
      const user = userEvent.setup();
      mockOpen.mockResolvedValue("/path/to/folder");

      render(<SettingsPage />);

      const browseButton = screen.getByRole("button", { name: /Browse/i });
      await user.click(browseButton);

      const folderInput = screen.getByLabelText(/Local Folder Path/i) as HTMLInputElement;

      await waitFor(() => {
        expect(folderInput.value).toBe("/path/to/folder");
      });
    });

    it("should not update folder path when dialog is cancelled", async () => {
      const user = userEvent.setup();
      mockOpen.mockResolvedValue(null);

      render(<SettingsPage />);

      const browseButton = screen.getByRole("button", { name: /Browse/i });
      await user.click(browseButton);

      const folderInput = screen.getByLabelText(/Local Folder Path/i) as HTMLInputElement;

      await waitFor(() => {
        expect(folderInput.value).toBe("");
      });
    });
  });

  describe("Back Button", () => {
    it("should call onBack when back button is clicked", async () => {
      const user = userEvent.setup();
      const mockOnBack = vi.fn();

      render(<SettingsPage onBack={mockOnBack} />);

      const backButton = screen.getByRole("button", { name: /Back to Main/i });
      await user.click(backButton);

      expect(mockOnBack).toHaveBeenCalledTimes(1);
    });
  });

  describe("Help Section", () => {
    it("should display help information", () => {
      render(<SettingsPage />);

      expect(screen.getByText("How to get AWS credentials:")).toBeInTheDocument();
      expect(screen.getByText(/Sign in to the AWS Management Console/i)).toBeInTheDocument();
    });
  });
});
