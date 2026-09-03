using System;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace Sweepie.App.CoreInterop;

public static class CoreNativeFfi
{
    private const string DllName = "sweepie_core";

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern IntPtr sweepie_version();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int sweepie_scan_path(string path, out IntPtr jsonOut);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern int sweepie_clean_json(string requestJson, out IntPtr resultOut);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void sweepie_free_string(IntPtr ptr);

    public static string? GetVersion()
    {
        try
        {
            IntPtr ptr = sweepie_version();
            return Marshal.PtrToStringAnsi(ptr);
        }
        catch
        {
            return null;
        }
    }

    public static ScanResult? ScanPathDirect(string path)
    {
        int code = sweepie_scan_path(path, out IntPtr jsonPtr);
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
            sweepie_free_string(jsonPtr);
        }
    }
}
