using Microsoft.UI.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Media;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Sweepie.App.CoreInterop;
using Sweepie.App.ViewModels;

namespace Sweepie.App;

public sealed partial class MainWindow : Window
{
    public MainViewModel ViewModel { get; }

    public MainWindow()
    {
        ViewModel = new MainViewModel();
        this.InitializeComponent();
        ViewModel.ConfirmCleanCallback = ShowCleanConfirmationDialogAsync;
    }

    private async Task<bool> ShowCleanConfirmationDialogAsync(List<CandidateViewModel> targets)
    {
        ulong totalBytes = 0;
        foreach (var t in targets) totalBytes += t.SizeBytes;
        string formattedSize = CoreCliBridge.FormatSize(totalBytes);

        var dialog = new ContentDialog
        {
            Title = "Confirm Space Reclamation",
            PrimaryButtonText = "Reclaim Space",
            CloseButtonText = "Cancel",
            DefaultButton = ContentDialogButton.Primary,
            XamlRoot = this.Content.XamlRoot,
        };

        var stack = new StackPanel { Spacing = 14, Width = 480 };

        var summaryText = new TextBlock
        {
            Text = $"Ready to reclaim {formattedSize} across {targets.Count} artifact location(s).",
            FontWeight = FontWeights.SemiBold,
            FontSize = 14,
        };
        stack.Children.Add(summaryText);

        var listBorder = new Border
        {
            BorderThickness = new Thickness(1),
            BorderBrush = (Brush)Application.Current.Resources["CardStrokeColorDefaultBrush"],
            Background = (Brush)Application.Current.Resources["LayerFillColorDefaultBrush"],
            CornerRadius = new CornerRadius(6),
            MaxHeight = 180,
            Padding = new Thickness(8),
        };
        var scroll = new ScrollViewer { VerticalScrollBarVisibility = ScrollBarVisibility.Auto };
        var listStack = new StackPanel { Spacing = 4 };
        foreach (var t in targets)
        {
            listStack.Children.Add(new TextBlock
            {
                Text = $"• {t.DisplayPath} ({t.HumanSize})",
                FontFamily = new FontFamily("Cascadia Code, Consolas, monospace"),
                FontSize = 11,
                TextTrimming = TextTrimming.CharacterEllipsis,
            });
        }
        scroll.Content = listStack;
        listBorder.Child = scroll;
        stack.Children.Add(listBorder);

        var recycleCheck = new CheckBox
        {
            Content = "Move to Recycle Bin (recommended)",
            IsChecked = ViewModel.UseRecycleBin,
        };
        stack.Children.Add(recycleCheck);

        var warningText = new TextBlock
        {
            Text = "⚠️ Warning: Permanent deletion cannot be undone. Files will bypass the Recycle Bin.",
            Foreground = (Brush)Application.Current.Resources["SystemFillColorCriticalBrush"],
            FontSize = 12,
            Visibility = recycleCheck.IsChecked == true ? Visibility.Collapsed : Visibility.Visible,
        };
        recycleCheck.Checked += (_, _) =>
        {
            warningText.Visibility = Visibility.Collapsed;
            ViewModel.UseRecycleBin = true;
        };
        recycleCheck.Unchecked += (_, _) =>
        {
            warningText.Visibility = Visibility.Visible;
            ViewModel.UseRecycleBin = false;
        };
        stack.Children.Add(warningText);

        dialog.Content = stack;

        var result = await dialog.ShowAsync();
        return result == ContentDialogResult.Primary;
    }
}

