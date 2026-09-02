using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;
using System;
using Undrift.App.ViewModels;

namespace Undrift.App;

public sealed partial class MainWindow : Window
{
    public MainViewModel ViewModel { get; }

    public MainWindow()
    {
        ViewModel = new MainViewModel();
        this.InitializeComponent();

        Title = "Undrift — Space Reclaiming for Developers";
    }
}

public class ScanButtonTextConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, string language)
    {
        return value is true ? "Scanning MFT..." : "Scan MFT";
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
    {
        throw new NotImplementedException();
    }
}
