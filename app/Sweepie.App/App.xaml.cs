using Microsoft.UI.Xaml;
using System;
using System.IO;

namespace Sweepie.App;

public partial class App : Application
{
    public static App CurrentApp { get; private set; } = null!;
    public static Window? MainWindowInstance { get; private set; }

    private static readonly string LogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments),
        "undrift",
        "app_debug.log");

    public App()
    {
        CurrentApp = this;

        AppDomain.CurrentDomain.ProcessExit += (s, e) =>
        {
            try
            {
                File.AppendAllText(LogPath, $"[ProcessExit] ExitCode = {Environment.ExitCode}\n");
            }
            catch { }
        };

        AppDomain.CurrentDomain.UnhandledException += (s, e) =>
        {
            try
            {
                File.AppendAllText(LogPath, $"[AppDomain.UnhandledException] {e.ExceptionObject}\n");
            }
            catch { }
        };

        this.UnhandledException += (s, e) =>
        {
            try
            {
                File.AppendAllText(LogPath, $"[Xaml.UnhandledException] {e.Message}\n{e.Exception}\n");
                e.Handled = false;
            }
            catch { }
        };

        try
        {
            this.InitializeComponent();
        }
        catch (Exception ex)
        {
            File.AppendAllText(LogPath, $"[InitializeComponent Exception] {ex}\n");
            throw;
        }
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        try
        {
            File.AppendAllText(LogPath, $"[OnLaunched] Launching at {DateTime.Now}\n");
            var window = new MainWindow();
            MainWindowInstance = window;

            window.Closed += (s, e) =>
            {
                File.AppendAllText(LogPath, "[Window Closed]\n");
            };

            window.Activate();

            var hWnd = WinRT.Interop.WindowNative.GetWindowHandle(window);
            var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hWnd);
            var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);
            if (appWindow != null)
            {
                appWindow.Title = "Sweepie — Find extra space on your PC";
                appWindow.Resize(new Windows.Graphics.SizeInt32(1150, 780));

                var iconPath = Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico");
                if (File.Exists(iconPath))
                {
                    appWindow.SetIcon(iconPath);
                }
            }
            File.AppendAllText(LogPath, $"[OnLaunched] Window activated successfully. Dispatcher active: {window.DispatcherQueue.HasThreadAccess}\n");
        }
        catch (Exception ex)
        {
            File.AppendAllText(LogPath, $"[OnLaunched Exception] {ex}\n");
            throw;
        }
    }
}
