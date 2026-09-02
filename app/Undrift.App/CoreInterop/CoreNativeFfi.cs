using System;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace Undrift.App.CoreInterop;

public static class CoreNativeFfi
{
    private const string DllName = "undrift_core";

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern IntPtr undrift_version();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int undrift_scan_path(string path, out IntPtr jsonOut);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int undrift_clean_json(string requestJson, out IntPtr resultOut);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void undrift_free_string(IntPtr ptr);

    public static string? GetVersion()
    {
        try
        {
            IntPtr ptr = undrift_version();
            return Marshal.PtrToStringAnsi(ptr);
        }
        catch
        {
            return null;
        }
    }

    public static ScanResult? ScanPathDirect(string path)
    {
        int code = undrift_scan_path(path, out IntPtr jsonPtr);
        if (code != 0 || jsonPtr == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            string? json = Marshal.PtrToStringUTF8(jsonPtr);
            return json != null ? JsonSerializer.Deserialize<ScanResult>(json) : null;
        }
        finally
        {
            undrift_free_string(jsonPtr);
        }
    }
}
