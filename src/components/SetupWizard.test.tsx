import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SetupWizard } from "./SetupWizard";

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

describe("SetupWizard", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockOpen.mockReset();
  });

  describe("Rendering", () => {
    it("should render the setup wizard with header", () => {
      render(<SetupWizard />);

      expect(screen.getByText("Setup Wizard")).toBeInTheDocument();
      expect(screen.getByText("Let's configure file-funeral for your first sync")).toBeInTheDocument();
    });

    it("should render progress bar", () => {
      render(<SetupWizard />);

      expect(screen.getByText("Step 1 of 5")).toBeInTheDocument();
      expect(screen.getByText("20%")).toBeInTheDocument();
    });

    it("should render navigation buttons", () => {
      render(<SetupWizard />);

      expect(screen.getByRole("button", { name: /Back/i })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /Next/i })).toBeInTheDocument();
    });

    it("should render skip button when onSkip prop is provided", () => {
      const mockOnSkip = vi.fn();
      render(<SetupWizard onSkip={mockOnSkip} />);

      expect(screen.getByRole("button", { name: /Skip Setup/i })).toBeInTheDocument();
    });

    it("should not render skip button when onSkip prop is not provided", () => {
      render(<SetupWizard />);

      expect(screen.queryByRole("button", { name: /Skip Setup/i })).not.toBeInTheDocument();
    });
  });

  describe("Step 1: Welcome Screen", () => {
    it("should render welcome screen content", () => {
      render(<SetupWizard />);

      expect(screen.getByText("Welcome to file-funeral")).toBeInTheDocument();
      expect(screen.getByText(/cloud-native desktop app/i)).toBeInTheDocument();
      expect(screen.getByText("Secure Cloud Backup")).toBeInTheDocument();
      expect(screen.getByText("Bidirectional Sync")).toBeInTheDocument();
      expect(screen.getByText("Conflict Resolution")).toBeInTheDocument();
    });

    it("should have back button disabled on first step", () => {
      render(<SetupWizard />);

      const backButton = screen.getByRole("button", { name: /Back/i });
      expect(backButton).toBeDisabled();
    });

    it("should have next button enabled on first step", () => {
      render(<SetupWizard />);

      const nextButton = screen.getByRole("button", { name: /Next/i });
      expect(nextButton).not.toBeDisabled();
    });
  });

  describe("Step Navigation", () => {
    it("should navigate to step 2 when next is clicked on step 1", async () => {
      const user = userEvent.setup();
      render(<SetupWizard />);

      const nextButton = screen.getByRole("button", { name: /Next/i });
      await user.click(nextButton);

      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
        expect(screen.getByText("Step 2 of 5")).toBeInTheDocument();
      });
    });

    it("should navigate back to step 1 when back is clicked on step 2", async () => {
      const user = userEvent.setup();
      render(<SetupWizard />);

      // Go to step 2
      await user.click(screen.getByRole("button", { name: /Next/i }));
      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      // Go back to step 1
      const backButton = screen.getByRole("button", { name: /Back/i });
      await user.click(backButton);

      await waitFor(() => {
        expect(screen.getByText("Welcome to file-funeral")).toBeInTheDocument();
        expect(screen.getByText("Step 1 of 5")).toBeInTheDocument();
      });
    });

    it("should update progress bar percentage as steps progress", async () => {
      const user = userEvent.setup();
      render(<SetupWizard />);

      // Step 1: 20%
      expect(screen.getByText("20%")).toBeInTheDocument();

      // Go to step 2: 40%
      await user.click(screen.getByRole("button", { name: /Next/i }));
      await waitFor(() => {
        expect(screen.getByText("40%")).toBeInTheDocument();
      });
    });
  });

  describe("Step 2: Credentials", () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      render(<SetupWizard />);
      await user.click(screen.getByRole("button", { name: /Next/i }));
      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });
    });

    it("should render credentials form", () => {
      expect(screen.getByLabelText(/Access Key ID/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Secret Access Key/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Region/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/S3 Bucket Name/i)).toBeInTheDocument();
    });

    it("should update form fields when typed", async () => {
      const user = userEvent.setup();

      const accessKeyInput = screen.getByLabelText(/Access Key ID/i) as HTMLInputElement;
      const secretKeyInput = screen.getByLabelText(/Secret Access Key/i) as HTMLInputElement;
      const bucketInput = screen.getByLabelText(/S3 Bucket Name/i) as HTMLInputElement;

      await user.type(accessKeyInput, "AKIAIOSFODNN7EXAMPLE");
      await user.type(secretKeyInput, "secretkey123");
      await user.type(bucketInput, "my-bucket");

      expect(accessKeyInput.value).toBe("AKIAIOSFODNN7EXAMPLE");
      expect(secretKeyInput.value).toBe("secretkey123");
      expect(bucketInput.value).toBe("my-bucket");
    });

    it("should disable next button when fields are empty", () => {
      const nextButton = screen.getByRole("button", { name: /Next/i });
      expect(nextButton).toBeDisabled();
    });

    it("should enable next button when all fields are filled", async () => {
      const user = userEvent.setup();

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const nextButton = screen.getByRole("button", { name: /Next/i });
      await waitFor(() => {
        expect(nextButton).not.toBeDisabled();
      });
    });

    it("should show validation error for invalid bucket name", async () => {
      const user = userEvent.setup();

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "Invalid_Bucket");

      const nextButton = screen.getByRole("button", { name: /Next/i });
      await user.click(nextButton);

      await waitFor(() => {
        expect(screen.getByText("Invalid bucket name format")).toBeInTheDocument();
      });
    });

    it("should save credentials when next is clicked", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({ success: true, message: "Saved" });

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const nextButton = screen.getByRole("button", { name: /Next/i });
      await user.click(nextButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("set_credentials", {
          request: {
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret_access_key: "secretkey123",
            region: "us-east-1",
            bucket_name: "my-bucket",
          },
        });
      });
    });

    it("should not proceed to next step if save fails", async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({ success: false, message: "Save failed" });

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const nextButton = screen.getByRole("button", { name: /Next/i });
      await user.click(nextButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalled();
      });

      // Should still be on step 2
      expect(screen.getByText("Step 2 of 5")).toBeInTheDocument();
      expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
    });
  });

  describe("Step 3: Folder Selection", () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue({ success: true, message: "Saved" });

      render(<SetupWizard />);

      // Navigate to step 2
      await user.click(screen.getByRole("button", { name: /Next/i }));
      await waitFor(() => {
        expect(screen.getByText("AWS Credentials")).toBeInTheDocument();
      });

      // Fill credentials and go to step 3
      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => {
        expect(screen.getByText("Select Sync Folder")).toBeInTheDocument();
      });
    });

    it("should render folder selection content", () => {
      expect(screen.getByText("Select Sync Folder")).toBeInTheDocument();
      expect(screen.getByLabelText(/Local Folder Path/i)).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /Browse/i })).toBeInTheDocument();
    });

    it("should disable next button when no folder is selected", () => {
      const nextButton = screen.getByRole("button", { name: /Next/i });
      expect(nextButton).toBeDisabled();
    });

    it("should open folder dialog when browse is clicked", async () => {
      const user = userEvent.setup();
      mockOpen.mockResolvedValue("/path/to/folder");

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

      const browseButton = screen.getByRole("button", { name: /Browse/i });
      await user.click(browseButton);

      const folderInput = screen.getByLabelText(/Local Folder Path/i) as HTMLInputElement;

      await waitFor(() => {
        expect(folderInput.value).toBe("/path/to/folder");
      });
    });

    it("should enable next button when folder is selected", async () => {
      const user = userEvent.setup();
      mockOpen.mockResolvedValue("/path/to/folder");

      const browseButton = screen.getByRole("button", { name: /Browse/i });
      await user.click(browseButton);

      const nextButton = screen.getByRole("button", { name: /Next/i });

      await waitFor(() => {
        expect(nextButton).not.toBeDisabled();
      });
    });
  });

  describe("Step 4: Connection Test", () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "set_credentials") {
          return Promise.resolve({ success: true, message: "Saved" });
        }
        return Promise.resolve({});
      });
      mockOpen.mockResolvedValue("/path/to/folder");

      render(<SetupWizard />);

      // Navigate to step 2
      await user.click(screen.getByRole("button", { name: /Next/i }));

      // Fill credentials and go to step 3
      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.selectOptions(screen.getByLabelText(/Region/i), "us-west-2");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");
      await user.click(screen.getByRole("button", { name: /Next/i }));

      // Select folder and go to step 4
      await waitFor(() => {
        expect(screen.getByText("Select Sync Folder")).toBeInTheDocument();
      });
      await user.click(screen.getByRole("button", { name: /Browse/i }));
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Next/i })).not.toBeDisabled();
      });
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => {
        expect(screen.getByText("Test Connection")).toBeInTheDocument();
      });
    });

    it("should render connection test content", () => {
      expect(screen.getByText("Test Connection")).toBeInTheDocument();
      expect(screen.getByText(/Verify your AWS credentials/i)).toBeInTheDocument();
    });

    it("should display configuration summary", () => {
      expect(screen.getByText("Access Key ID:")).toBeInTheDocument();
      expect(screen.getByText("Region:")).toBeInTheDocument();
      expect(screen.getByText("Bucket:")).toBeInTheDocument();
      expect(screen.getByText("Sync Folder:")).toBeInTheDocument();
      expect(screen.getByText("us-west-2")).toBeInTheDocument();
      expect(screen.getByText("my-bucket")).toBeInTheDocument();
    });

    it("should disable next button before connection test", () => {
      const nextButton = screen.getByRole("button", { name: /Next/i });
      expect(nextButton).toBeDisabled();
    });

    it.skip("should test connection when next is clicked", async () => {
      const user = userEvent.setup();

      const nextButton = screen.getByRole("button", { name: /Next/i });

      // Clear previous call history and set up mock for test_connection
      mockInvoke.mockClear();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "test_connection") {
          return Promise.resolve({
            connected: true,
            message: "Connected successfully",
            region: "us-west-2",
            bucket_name: "my-bucket",
          });
        }
        return Promise.resolve({});
      });

      await user.click(nextButton);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("test_connection");
      });
    });

    it.skip("should enable next button when connection test succeeds", async () => {
      const user = userEvent.setup();

      const nextButton = screen.getByRole("button", { name: /Next/i });

      // Clear previous call history and set up mock for test_connection
      mockInvoke.mockClear();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "test_connection") {
          return Promise.resolve({
            connected: true,
            message: "Connected successfully",
            region: "us-west-2",
            bucket_name: "my-bucket",
          });
        }
        return Promise.resolve({});
      });

      await user.click(nextButton);

      await waitFor(() => {
        expect(screen.getByText("Connected successfully")).toBeInTheDocument();
      });

      await waitFor(() => {
        expect(nextButton).not.toBeDisabled();
      });
    });

    it.skip("should not proceed when connection test fails", async () => {
      const user = userEvent.setup();

      const nextButton = screen.getByRole("button", { name: /Next/i });

      // Clear previous call history and set up mock for test_connection
      mockInvoke.mockClear();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "test_connection") {
          return Promise.resolve({
            connected: false,
            message: "Connection failed",
          });
        }
        return Promise.resolve({});
      });

      await user.click(nextButton);

      await waitFor(() => {
        expect(screen.getByText("Connection failed")).toBeInTheDocument();
      });

      // Should still be on step 4
      expect(screen.getByText("Step 4 of 5")).toBeInTheDocument();
      expect(nextButton).toBeDisabled();
    });
  });

  describe.skip("Step 5: Complete", () => {
    beforeEach(async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "set_credentials") {
          return Promise.resolve({ success: true, message: "Saved" });
        }
        if (cmd === "test_connection") {
          return Promise.resolve({
            connected: true,
            message: "Connected successfully",
            region: "us-west-2",
            bucket_name: "my-bucket",
          });
        }
        return Promise.resolve({});
      });
      mockOpen.mockResolvedValue("/path/to/folder");

      render(<SetupWizard />);

      // Navigate through all steps
      await user.click(screen.getByRole("button", { name: /Next/i })); // Step 1 -> 2

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.selectOptions(screen.getByLabelText(/Region/i), "us-west-2");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");
      await user.click(screen.getByRole("button", { name: /Next/i })); // Step 2 -> 3

      await waitFor(() => {
        expect(screen.getByText("Select Sync Folder")).toBeInTheDocument();
      });
      await user.click(screen.getByRole("button", { name: /Browse/i }));
      await waitFor(() => {
        expect(screen.getByRole("button", { name: /Next/i })).not.toBeDisabled();
      });
      await user.click(screen.getByRole("button", { name: /Next/i })); // Step 3 -> 4

      await waitFor(() => {
        expect(screen.getByText("Test Connection")).toBeInTheDocument();
      });
      await user.click(screen.getByRole("button", { name: /Next/i })); // Step 4 -> 5 (triggers test)

      await waitFor(() => {
        expect(screen.getByText("Setup Complete!")).toBeInTheDocument();
      });
    });

    it("should render completion screen", () => {
      expect(screen.getByText("Setup Complete!")).toBeInTheDocument();
      expect(screen.getByText(/now configured and ready to use/i)).toBeInTheDocument();
    });

    it("should display configuration summary", () => {
      expect(screen.getByText("Configuration Summary:")).toBeInTheDocument();
      expect(screen.getByText("AWS Region:")).toBeInTheDocument();
      expect(screen.getByText("S3 Bucket:")).toBeInTheDocument();
      expect(screen.getByText("Sync Folder:")).toBeInTheDocument();
    });

    it("should show Get Started button instead of Next", () => {
      expect(screen.getByRole("button", { name: /Get Started/i })).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: /Next/i })).not.toBeInTheDocument();
    });

    it("should call onComplete when Get Started is clicked", async () => {
      const user = userEvent.setup();
      const mockOnComplete = vi.fn();

      // Re-render with onComplete prop
      render(<SetupWizard onComplete={mockOnComplete} />);
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "set_credentials") {
          return Promise.resolve({ success: true, message: "Saved" });
        }
        if (cmd === "test_connection") {
          return Promise.resolve({
            connected: true,
            message: "Connected successfully",
            region: "us-west-2",
            bucket_name: "my-bucket",
          });
        }
        return Promise.resolve({});
      });
      mockOpen.mockResolvedValue("/path/to/folder");

      // Navigate through all steps quickly
      await user.click(screen.getAllByRole("button", { name: /Next/i })[0]);

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Select Sync Folder"));
      await user.click(screen.getByRole("button", { name: /Browse/i }));
      await waitFor(() => expect(screen.getByRole("button", { name: /Next/i })).not.toBeDisabled());
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Test Connection"));
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Setup Complete!"));

      const getStartedButton = screen.getByRole("button", { name: /Get Started/i });
      await user.click(getStartedButton);

      expect(mockOnComplete).toHaveBeenCalledTimes(1);
    });

    it("should not show skip button on final step", () => {
      expect(screen.queryByRole("button", { name: /Skip Setup/i })).not.toBeInTheDocument();
    });

    it("should show 100% progress", () => {
      expect(screen.getByText("Step 5 of 5")).toBeInTheDocument();
      expect(screen.getByText("100%")).toBeInTheDocument();
    });
  });

  describe("Skip Button", () => {
    it("should call onSkip when skip button is clicked", async () => {
      const user = userEvent.setup();
      const mockOnSkip = vi.fn();

      render(<SetupWizard onSkip={mockOnSkip} />);

      const skipButton = screen.getByRole("button", { name: /Skip Setup/i });
      await user.click(skipButton);

      expect(mockOnSkip).toHaveBeenCalledTimes(1);
    });

    it.skip("should show skip button on all steps except step 5", async () => {
      const user = userEvent.setup();
      const mockOnSkip = vi.fn();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "set_credentials") {
          return Promise.resolve({ success: true, message: "Saved" });
        }
        if (cmd === "test_connection") {
          return Promise.resolve({
            connected: true,
            message: "Connected successfully",
            region: "us-west-2",
            bucket_name: "my-bucket",
          });
        }
        return Promise.resolve({});
      });
      mockOpen.mockResolvedValue("/path/to/folder");

      render(<SetupWizard onSkip={mockOnSkip} />);

      // Step 1: Skip button should be visible
      expect(screen.getByRole("button", { name: /Skip Setup/i })).toBeInTheDocument();

      // Go to step 2
      await user.click(screen.getByRole("button", { name: /Next/i }));
      await waitFor(() => screen.getByText("AWS Credentials"));
      expect(screen.getByRole("button", { name: /Skip Setup/i })).toBeInTheDocument();

      // Go to step 5
      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Select Sync Folder"));
      await user.click(screen.getByRole("button", { name: /Browse/i }));
      await waitFor(() => expect(screen.getByRole("button", { name: /Next/i })).not.toBeDisabled());
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Test Connection"));
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Setup Complete!"));

      // Step 5: Skip button should NOT be visible
      expect(screen.queryByRole("button", { name: /Skip Setup/i })).not.toBeInTheDocument();
    });
  });

  describe.skip("Button States", () => {
    it("should disable buttons while saving credentials", async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "set_credentials") {
          return new Promise((resolve) => setTimeout(() => resolve({ success: true }), 100));
        }
        return Promise.resolve({});
      });

      render(<SetupWizard />);

      await user.click(screen.getByRole("button", { name: /Next/i }));

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");

      const nextButton = screen.getByRole("button", { name: /Next/i });
      const backButton = screen.getByRole("button", { name: /Back/i });

      await user.click(nextButton);

      // Wait for buttons to be disabled
      await waitFor(() => {
        expect(nextButton).toBeDisabled();
        expect(backButton).toBeDisabled();
      });

      // Wait for buttons to be enabled again
      await waitFor(() => {
        expect(nextButton).not.toBeDisabled();
        expect(backButton).not.toBeDisabled();
      });
    });

    it("should disable buttons while testing connection", async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation((cmd) => {
        if (cmd === "set_credentials") {
          return Promise.resolve({ success: true, message: "Saved" });
        }
        if (cmd === "test_connection") {
          return new Promise((resolve) =>
            setTimeout(
              () =>
                resolve({
                  connected: true,
                  message: "Connected successfully",
                  region: "us-west-2",
                  bucket_name: "my-bucket",
                }),
              100
            )
          );
        }
        return Promise.resolve({});
      });
      mockOpen.mockResolvedValue("/path/to/folder");

      render(<SetupWizard />);

      // Navigate to step 4
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await user.type(screen.getByLabelText(/Access Key ID/i), "AKIAIOSFODNN7EXAMPLE");
      await user.type(screen.getByLabelText(/Secret Access Key/i), "secretkey123");
      await user.type(screen.getByLabelText(/S3 Bucket Name/i), "my-bucket");
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Select Sync Folder"));
      await user.click(screen.getByRole("button", { name: /Browse/i }));
      await waitFor(() => expect(screen.getByRole("button", { name: /Next/i })).not.toBeDisabled());
      await user.click(screen.getByRole("button", { name: /Next/i }));

      await waitFor(() => screen.getByText("Test Connection"));

      const nextButton = screen.getByRole("button", { name: /Next/i });
      const backButton = screen.getByRole("button", { name: /Back/i });

      await user.click(nextButton);

      // Wait for buttons to be disabled
      await waitFor(() => {
        expect(nextButton).toBeDisabled();
        expect(backButton).toBeDisabled();
      });

      // Wait for buttons to be enabled again
      await waitFor(() => {
        expect(nextButton).not.toBeDisabled();
        expect(backButton).not.toBeDisabled();
      });
    });
  });
});
