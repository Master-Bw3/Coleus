package mod.master_bw3.coleus.internal

internal class BookConfig (
    val css: List<String>?,
    val themes: List<ThemeEntry>?,
    val default_theme: String?,
    val spoiler_advancements: List<String>?
) {
    internal class ThemeEntry(val id: String, val location: String)
}