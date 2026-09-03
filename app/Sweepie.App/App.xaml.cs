using Microsoft.UI.Xaml;
using System;
using System.IO;

namespace Sweepie.App;

public partial class App : Application
{
    public static App CurrentApp { get; private set; } = null!;
    public static Window? MainWindowInstance { get; private set; }

    public App()
    {
        CurrentApp = this;
        this.InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        var window = new MainWindow();
        MainWindowInstance = window;
        window.Activate();

        var hWnd = WinRT.Interop.WindowNative.GetWindowHandle(window);
        var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hWnd);
        var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);
        if (appWindow != null)
        {
            appWindow.Title = "Sweepie — Space Reclaiming for Developers";
            appWindow.Resize(new Windows.Graphics.SizeInt32(1150, 780));

            var iconPath = Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico");
            if (File.Exists(iconPath))
            {
                appWindow.SetIcon(iconPath);
            }
        }
    }
}
