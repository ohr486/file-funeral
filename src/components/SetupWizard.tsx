import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Sparkles,
  Key,
  Folder,
  CheckCircle,
  CheckCircle2,
  AlertCircle,
  ArrowRight,
  ArrowLeft
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Progress } from "@/components/ui/progress";
import { toast } from "sonner";
import { SetCredentialsRequest, CredentialsResponse, ConnectionTestResponse } from "@/types";

interface SetupWizardProps {
  onComplete?: () => void;
  onSkip?: () => void;
}

type SetupStep = 1 | 2 | 3 | 4 | 5;
type TestStatus = "idle" | "testing" | "success" | "error";

export function SetupWizard({ onComplete, onSkip }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState<SetupStep>(1);

  // Step 2: Credentials
  const [accessKeyId, setAccessKeyId] = useState("");
  const [secretAccessKey, setSecretAccessKey] = useState("");
  const [region, setRegion] = useState("us-east-1");
  const [bucketName, setBucketName] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSaving, setIsSaving] = useState(false);

  // Step 3: Folder
  const [syncFolder, setSyncFolder] = useState("");

  // Step 4: Test
  const [testStatus, setTestStatus] = useState<TestStatus>("idle");
  const [testMessage, setTestMessage] = useState("");

  const progressPercentage = (currentStep / 5) * 100;

  const validateCredentials = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!accessKeyId.trim()) {
      newErrors.accessKeyId = "Access Key ID is required";
    }

    if (!secretAccessKey.trim()) {
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

  const handleSaveCredentials = async () => {
    if (!validateCredentials()) {
      toast.error("Please fix the validation errors");
      return false;
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
        toast.success("Credentials saved successfully");
        return true;
      } else {
        toast.error("Failed to save credentials", {
          description: response.message,
        });
        return false;
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      toast.error("Error saving credentials", {
        description: errorMessage,
      });
      return false;
    } finally {
      setIsSaving(false);
    }
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

  const handleTestConnection = async () => {
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
        return true;
      } else {
        setTestStatus("error");
        setTestMessage(response.message);
        toast.error("Connection failed", {
          description: response.message,
        });
        return false;
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      setTestStatus("error");
      setTestMessage(errorMessage);
      toast.error("Connection test failed", {
        description: errorMessage,
      });
      return false;
    }
  };

  const handleNext = async () => {
    if (currentStep === 2) {
      const saved = await handleSaveCredentials();
      if (!saved) return;
    }

    if (currentStep === 4) {
      const connected = await handleTestConnection();
      if (!connected) return;
    }

    if (currentStep < 5) {
      setCurrentStep((currentStep + 1) as SetupStep);
    }
  };

  const handleBack = () => {
    if (currentStep > 1) {
      setCurrentStep((currentStep - 1) as SetupStep);
    }
  };

  const handleFinish = () => {
    onComplete?.();
  };

  const canProceed = () => {
    switch (currentStep) {
      case 1:
        return true;
      case 2:
        return accessKeyId && secretAccessKey && region && bucketName;
      case 3:
        return syncFolder !== "";
      case 4:
        return true;
      case 5:
        return true;
      default:
        return false;
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-50 to-slate-100 p-6">
      <Card className="w-full max-w-3xl">
        <CardHeader>
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <Sparkles className="h-8 w-8 text-primary" />
              <div>
                <CardTitle className="text-2xl">Setup Wizard</CardTitle>
                <CardDescription>
                  Let's configure file-funeral for your first sync
                </CardDescription>
              </div>
            </div>
            {onSkip && currentStep < 5 && (
              <Button variant="ghost" onClick={onSkip} size="sm">
                Skip Setup
              </Button>
            )}
          </div>
          <Progress value={progressPercentage} className="h-2" />
          <div className="flex justify-between text-xs text-muted-foreground mt-2">
            <span>Step {currentStep} of 5</span>
            <span>{Math.round(progressPercentage)}%</span>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Step 1: Welcome */}
          {currentStep === 1 && (
            <div className="space-y-6 py-8 text-center">
              <div className="flex justify-center">
                <div className="rounded-full bg-primary/10 p-6">
                  <Sparkles className="h-16 w-16 text-primary" />
                </div>
              </div>
              <div className="space-y-2">
                <h2 className="text-3xl font-bold">Welcome to file-funeral</h2>
                <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                  A cloud-native desktop app for backing up and syncing files between your
                  local storage and cloud providers.
                </p>
              </div>
              <div className="space-y-3 text-left max-w-md mx-auto">
                <div className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 text-primary mt-0.5 shrink-0" />
                  <div>
                    <p className="font-medium">Secure Cloud Backup</p>
                    <p className="text-sm text-muted-foreground">
                      Store your files safely in AWS S3 with encryption
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 text-primary mt-0.5 shrink-0" />
                  <div>
                    <p className="font-medium">Bidirectional Sync</p>
                    <p className="text-sm text-muted-foreground">
                      Keep your files in sync across multiple devices
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 text-primary mt-0.5 shrink-0" />
                  <div>
                    <p className="font-medium">Conflict Resolution</p>
                    <p className="text-sm text-muted-foreground">
                      Automatic conflict handling with "keep both" strategy
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Step 2: Credentials */}
          {currentStep === 2 && (
            <div className="space-y-6">
              <div className="flex items-center gap-3 pb-4">
                <div className="rounded-full bg-primary/10 p-3">
                  <Key className="h-6 w-6 text-primary" />
                </div>
                <div>
                  <h3 className="text-xl font-semibold">AWS Credentials</h3>
                  <p className="text-sm text-muted-foreground">
                    Enter your AWS credentials to connect to S3
                  </p>
                </div>
              </div>

              <div className="space-y-4">
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
                    placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
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

                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Your credentials will be securely stored in your system keychain.
                  </AlertDescription>
                </Alert>
              </div>
            </div>
          )}

          {/* Step 3: Folder Selection */}
          {currentStep === 3 && (
            <div className="space-y-6">
              <div className="flex items-center gap-3 pb-4">
                <div className="rounded-full bg-primary/10 p-3">
                  <Folder className="h-6 w-6 text-primary" />
                </div>
                <div>
                  <h3 className="text-xl font-semibold">Select Sync Folder</h3>
                  <p className="text-sm text-muted-foreground">
                    Choose the local folder you want to synchronize
                  </p>
                </div>
              </div>

              <div className="space-y-4">
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

                {syncFolder && (
                  <Alert>
                    <CheckCircle2 className="h-4 w-4" />
                    <AlertDescription className="break-all">
                      <span className="font-medium">Selected:</span>{" "}
                      <span className="text-xs">{syncFolder}</span>
                    </AlertDescription>
                  </Alert>
                )}

                {!syncFolder && (
                  <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                      Please select a folder to continue
                    </AlertDescription>
                  </Alert>
                )}
              </div>
            </div>
          )}

          {/* Step 4: Connection Test */}
          {currentStep === 4 && (
            <div className="space-y-6">
              <div className="flex items-center gap-3 pb-4">
                <div className="rounded-full bg-primary/10 p-3">
                  <CheckCircle className="h-6 w-6 text-primary" />
                </div>
                <div>
                  <h3 className="text-xl font-semibold">Test Connection</h3>
                  <p className="text-sm text-muted-foreground">
                    Verify your AWS credentials and S3 bucket access
                  </p>
                </div>
              </div>

              <div className="space-y-4">
                <div className="bg-muted/50 rounded-lg p-6 space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-sm font-medium">Access Key ID:</span>
                    <span className="text-sm text-muted-foreground font-mono">
                      {accessKeyId.substring(0, 12)}...
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm font-medium">Region:</span>
                    <span className="text-sm text-muted-foreground">{region}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm font-medium">Bucket:</span>
                    <span className="text-sm text-muted-foreground">{bucketName}</span>
                  </div>
                  <div className="flex flex-col gap-1">
                    <span className="text-sm font-medium">Sync Folder:</span>
                    <span className="text-xs text-muted-foreground break-all">
                      {syncFolder}
                    </span>
                  </div>
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

                {testStatus === "idle" && (
                  <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                      Click "Next" to test the connection to your S3 bucket
                    </AlertDescription>
                  </Alert>
                )}
              </div>
            </div>
          )}

          {/* Step 5: Complete */}
          {currentStep === 5 && (
            <div className="space-y-6 py-8 text-center">
              <div className="flex justify-center">
                <div className="rounded-full bg-green-100 p-6">
                  <CheckCircle className="h-16 w-16 text-green-600" />
                </div>
              </div>
              <div className="space-y-2">
                <h2 className="text-3xl font-bold">Setup Complete!</h2>
                <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                  Your file-funeral is now configured and ready to use.
                </p>
              </div>
              <div className="bg-muted/50 rounded-lg p-6 space-y-2 max-w-md mx-auto">
                <p className="text-sm font-medium">Configuration Summary:</p>
                <div className="space-y-1 text-sm text-muted-foreground text-left">
                  <div className="flex justify-between">
                    <span>AWS Region:</span>
                    <span className="font-mono">{region}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>S3 Bucket:</span>
                    <span className="font-mono">{bucketName}</span>
                  </div>
                  <div className="flex flex-col gap-1">
                    <span>Sync Folder:</span>
                    <span className="font-mono text-xs break-all" title={syncFolder}>
                      {syncFolder}
                    </span>
                  </div>
                </div>
              </div>
              <Alert>
                <CheckCircle2 className="h-4 w-4" />
                <AlertDescription>
                  You can change these settings anytime from the Settings page
                </AlertDescription>
              </Alert>
            </div>
          )}

          {/* Navigation Buttons */}
          <div className="flex justify-between pt-6 border-t">
            <Button
              variant="outline"
              onClick={handleBack}
              disabled={currentStep === 1 || isSaving || testStatus === "testing"}
            >
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back
            </Button>

            {currentStep < 5 ? (
              <Button
                onClick={handleNext}
                disabled={!canProceed() || isSaving || testStatus === "testing"}
              >
                {isSaving ? "Saving..." : testStatus === "testing" ? "Testing..." : "Next"}
                <ArrowRight className="ml-2 h-4 w-4" />
              </Button>
            ) : (
              <Button onClick={handleFinish}>
                Get Started
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
