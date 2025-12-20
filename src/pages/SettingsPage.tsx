import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Settings, CheckCircle2, AlertCircle, Folder } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { toast } from "sonner";
import { SetCredentialsRequest, CredentialsResponse, ConnectionTestResponse, GetCredentialsResponse } from "@/types";

interface SettingsPageProps {
  onBack?: () => void;
  onSetupWizard?: () => void;
}

type TestStatus = "idle" | "testing" | "success" | "error";

export function SettingsPage({ onBack, onSetupWizard }: SettingsPageProps) {
  const [accessKeyId, setAccessKeyId] = useState("");
  const [secretAccessKey, setSecretAccessKey] = useState("");
  const [region, setRegion] = useState("us-east-1");
  const [bucketName, setBucketName] = useState("");
  const [syncFolder, setSyncFolder] = useState("");
  const [testStatus, setTestStatus] = useState<TestStatus>("idle");
  const [testMessage, setTestMessage] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [credentialsSaved, setCredentialsSaved] = useState(false);

  // Load existing credentials when component mounts
  useEffect(() => {
    const loadCredentials = async () => {
      try {
        const response = await invoke<GetCredentialsResponse>("get_credentials");

        if (response.has_credentials) {
          if (response.access_key_id) setAccessKeyId(response.access_key_id);
          if (response.region) setRegion(response.region);
          if (response.bucket_name) setBucketName(response.bucket_name);
          setCredentialsSaved(true);
          // Secret key is not returned for security, so it remains empty
        }
      } catch (error) {
        console.error("Failed to load credentials:", error);
        // Silently fail - user can enter credentials manually
      }
    };

    loadCredentials();
  }, []);

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!accessKeyId.trim()) {
      newErrors.accessKeyId = "Access Key ID is required";
    }

    // Secret key is only required if we don't have saved credentials
    if (!secretAccessKey.trim() && !credentialsSaved) {
      newErrors.secretAccessKey = "Secret Access Key is required";
    }

    if (!region.trim()) {
      newErrors.region = "Region is required";
    }

    if (!bucketName.trim()) {
      newErrors.bucketName = "Bucket name is required";
    } else if (!/^[a-z0-9][a-z0-9.-]*[a-z0-9]$/.test(bucketName)) {
      newErrors.bucketName = "Invalid bucket name format";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select folder to sync",
      });

      if (selected && typeof selected === "string") {
        setSyncFolder(selected);
        toast.success("Folder selected", {
          description: selected,
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      toast.error("Failed to select folder", {
        description: errorMessage,
      });
    }
  };

  const handleSaveCredentials = async () => {
    if (!validateForm()) {
      toast.error("Please fix the validation errors");
      return;
    }

    setIsSaving(true);
    try {
      const request: SetCredentialsRequest = {
        access_key_id: accessKeyId.trim(),
        secret_access_key: secretAccessKey.trim(),
        region: region.trim(),
        bucket_name: bucketName.trim(),
      };

      const response = await invoke<CredentialsResponse>("set_credentials", {
        request,
      });

      if (response.success) {
        setCredentialsSaved(true);
        toast.success("Credentials saved successfully", {
          description: "Your AWS credentials have been securely stored",
        });
        // Clear sensitive data from state after saving
        setSecretAccessKey("");

        // Automatically test connection after successful save
        setTimeout(() => {
          handleTestConnection();
        }, 500);
      } else {
        toast.error("Failed to save credentials", {
          description: response.message,
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      toast.error("Error saving credentials", {
        description: errorMessage,
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    if (!credentialsSaved) {
      toast.error("Please save credentials first", {
        description: "You need to save your AWS credentials before testing the connection",
      });
      return;
    }

    setTestStatus("testing");
    setTestMessage("");

    try {
      const response = await invoke<ConnectionTestResponse>("test_connection");

      if (response.connected) {
        setTestStatus("success");
        setTestMessage(response.message);
        toast.success("Connection successful", {
          description: `Connected to ${response.bucket_name} in ${response.region}`,
        });
      } else {
        setTestStatus("error");
        setTestMessage(response.message);
        toast.error("Connection failed", {
          description: response.message,
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      setTestStatus("error");
      setTestMessage(errorMessage);
      toast.error("Connection test failed", {
        description: errorMessage,
      });
    }
  };

  return (
    <div className="container mx-auto p-6 max-w-4xl">
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Settings className="h-8 w-8" />
            <div>
              <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
              <p className="text-muted-foreground">
                Configure AWS credentials and sync preferences
              </p>
            </div>
          </div>
          {onBack && (
            <Button onClick={onBack} variant="outline">
              Back to Main
            </Button>
          )}
        </div>

        <Separator />

        {/* AWS Credentials Section */}
        <Card>
          <CardHeader>
            <CardTitle>AWS Credentials</CardTitle>
            <CardDescription>
              Enter your AWS credentials to connect to S3. Credentials are stored
              securely in your system keychain.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="accessKeyId">
                Access Key ID <span className="text-red-500">*</span>
              </Label>
              <Input
                id="accessKeyId"
                type="text"
                placeholder="AKIAIOSFODNN7EXAMPLE"
                value={accessKeyId}
                onChange={(e) => {
                  setAccessKeyId(e.target.value);
                  if (errors.accessKeyId) {
                    setErrors({ ...errors, accessKeyId: "" });
                  }
                }}
                className={errors.accessKeyId ? "border-red-500" : ""}
              />
              {errors.accessKeyId && (
                <p className="text-sm text-red-500">{errors.accessKeyId}</p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="secretAccessKey">
                Secret Access Key <span className="text-red-500">*</span>
              </Label>
              <Input
                id="secretAccessKey"
                type="password"
                placeholder={
                  credentialsSaved && !secretAccessKey
                    ? "••••••••  (Leave empty to keep existing)"
                    : "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
                }
                value={secretAccessKey}
                onChange={(e) => {
                  setSecretAccessKey(e.target.value);
                  if (errors.secretAccessKey) {
                    setErrors({ ...errors, secretAccessKey: "" });
                  }
                }}
                className={errors.secretAccessKey ? "border-red-500" : ""}
              />
              {errors.secretAccessKey && (
                <p className="text-sm text-red-500">{errors.secretAccessKey}</p>
              )}
              {credentialsSaved && !secretAccessKey && (
                <p className="text-sm text-muted-foreground">
                  Saved credentials found. Leave empty to keep existing secret key.
                </p>
              )}
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="region">
                  Region <span className="text-red-500">*</span>
                </Label>
                <select
                  id="region"
                  value={region}
                  onChange={(e) => {
                    setRegion(e.target.value);
                    if (errors.region) {
                      setErrors({ ...errors, region: "" });
                    }
                  }}
                  className={`flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
                    errors.region ? "border-red-500" : ""
                  }`}
                >
                  <option value="us-east-1">US East (N. Virginia)</option>
                  <option value="us-east-2">US East (Ohio)</option>
                  <option value="us-west-1">US West (N. California)</option>
                  <option value="us-west-2">US West (Oregon)</option>
                  <option value="ap-northeast-1">Asia Pacific (Tokyo)</option>
                  <option value="ap-northeast-2">Asia Pacific (Seoul)</option>
                  <option value="ap-southeast-1">Asia Pacific (Singapore)</option>
                  <option value="ap-southeast-2">Asia Pacific (Sydney)</option>
                  <option value="eu-central-1">Europe (Frankfurt)</option>
                  <option value="eu-west-1">Europe (Ireland)</option>
                  <option value="eu-west-2">Europe (London)</option>
                </select>
                {errors.region && (
                  <p className="text-sm text-red-500">{errors.region}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="bucketName">
                  S3 Bucket Name <span className="text-red-500">*</span>
                </Label>
                <Input
                  id="bucketName"
                  type="text"
                  placeholder="my-bucket-name"
                  value={bucketName}
                  onChange={(e) => {
                    setBucketName(e.target.value);
                    if (errors.bucketName) {
                      setErrors({ ...errors, bucketName: "" });
                    }
                  }}
                  className={errors.bucketName ? "border-red-500" : ""}
                />
                {errors.bucketName && (
                  <p className="text-sm text-red-500">{errors.bucketName}</p>
                )}
              </div>
            </div>

            <div className="flex gap-3 pt-2">
              <Button
                onClick={handleSaveCredentials}
                disabled={isSaving}
                className="flex-1"
              >
                {isSaving ? "Saving..." : "Save Credentials"}
              </Button>
              <Button
                onClick={handleTestConnection}
                variant="outline"
                disabled={testStatus === "testing" || !credentialsSaved}
                className="flex-1"
                title={
                  !credentialsSaved
                    ? "Save credentials first to test connection"
                    : "Test connection to AWS S3"
                }
              >
                {testStatus === "testing" ? "Testing..." : "Test Connection"}
              </Button>
            </div>

            {testStatus !== "idle" && (
              <Alert
                variant={testStatus === "success" ? "default" : "destructive"}
              >
                {testStatus === "success" ? (
                  <CheckCircle2 className="h-4 w-4" />
                ) : (
                  <AlertCircle className="h-4 w-4" />
                )}
                <AlertDescription>{testMessage}</AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>

        {/* Sync Folder Section */}
        <Card>
          <CardHeader>
            <CardTitle>Sync Folder</CardTitle>
            <CardDescription>
              Select the local folder you want to synchronize with S3
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="syncFolder">Local Folder Path</Label>
              <div className="flex gap-2">
                <Input
                  id="syncFolder"
                  type="text"
                  placeholder="No folder selected"
                  value={syncFolder}
                  readOnly
                  className="flex-1"
                />
                <Button onClick={handleSelectFolder} variant="outline">
                  <Folder className="mr-2 h-4 w-4" />
                  Browse
                </Button>
              </div>
              {syncFolder && (
                <p className="text-sm text-muted-foreground">
                  Files in this folder will be synchronized with S3
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Help Section */}
        <Card>
          <CardHeader>
            <CardTitle>Need Help?</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-muted-foreground">
            <p>
              <strong>How to get AWS credentials:</strong>
            </p>
            <ol className="list-decimal list-inside space-y-1 ml-2">
              <li>Sign in to the AWS Management Console</li>
              <li>Navigate to IAM (Identity and Access Management)</li>
              <li>Create a new user or select an existing user</li>
              <li>Attach the AmazonS3FullAccess policy</li>
              <li>Generate access keys and copy them here</li>
            </ol>
          </CardContent>
        </Card>

        {/* Setup Wizard Section */}
        {onSetupWizard && (
          <Card>
            <CardHeader>
              <CardTitle>Setup Wizard</CardTitle>
              <CardDescription>
                Run the guided setup wizard to configure your settings step by step
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button onClick={onSetupWizard} variant="outline" className="w-full">
                Run Setup Wizard
              </Button>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
