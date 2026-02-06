package mod.master_bw3.coleus.internal

import com.google.gson.GsonBuilder
import io.wispforest.lavender.Lavender
import io.wispforest.lavender.book.Book
import io.wispforest.lavender.book.Category
import io.wispforest.lavender.book.Entry
import io.wispforest.lavender.md.compiler.BookCompiler.ComponentSource
import io.wispforest.lavendermd.MarkdownProcessor
import io.wispforest.lavendermd.feature.*
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.parsing.UIModel
import j2html.TagCreator.*
import j2html.rendering.FlatHtml
import j2html.tags.UnescapedText
import j2html.tags.specialized.ATag
import j2html.tags.specialized.DivTag
import j2html.tags.specialized.FormTag
import j2html.tags.specialized.LinkTag
import j2html.tags.specialized.OlTag
import j2html.tags.specialized.ScriptTag
import j2html.tags.specialized.SelectTag
import mod.master_bw3.coleus.Base16Theme
import mod.master_bw3.coleus.PageContext
import mod.master_bw3.coleus.ColeusClient
import mod.master_bw3.coleus.SearchEntry
import mod.master_bw3.coleus.ThemeRegistry
import mod.master_bw3.coleus.lavender.compiler.HtmlPageCompiler
import mod.master_bw3.coleus.lavender.feature.*
import mod.master_bw3.coleus.mixin.client.LavenderBookScreenAccessor
import net.fabricmc.loader.api.FabricLoader
import net.minecraft.client.MinecraftClient
import net.minecraft.util.Identifier
import java.nio.file.Path
import java.util.stream.Collectors
import kotlin.collections.sortedWith
import kotlin.io.path.outputStream
import kotlin.io.path.relativeTo
import kotlin.io.path.writer
import kotlin.jvm.optionals.getOrNull

internal class HtmlBookGenerator(private val book: Book) {

    private val categoryComparator = compareBy {  it: Category -> it.ordinal }.thenBy { it.title.lowercase() }
    private val entryComparator = compareBy {  it: Entry -> it.ordinal }.thenBy { it.title.lowercase() }

    private val bookDir: Path =
        FabricLoader.getInstance().gameDir.resolve(ColeusClient.NAME).resolve(book.id().namespace)
            .resolve(book.id().path)
    private val assetDir: Path = bookDir.resolve("assets")

    private val searchEntries: MutableList<SearchEntry> = mutableListOf()

    private val config: BookConfig? by lazy {
        val configResource = MinecraftClient.getInstance().resourceManager.getResource(Identifier.of(book.id().namespace, "coleus/book.json"))

        val builder = GsonBuilder().create()

       configResource.map {
           builder.fromJson(
               configResource.get().reader.lines().collect(Collectors.joining("\n")),
               BookConfig::class.java
           )
       }.getOrNull()
    }

    internal fun generate(): Path {
        bookDir.toFile().deleteRecursively()
        includeThemes()

        val orderedPages = mutableListOf<Page>()

        book.landingPage()?.let { entry ->
            val path = bookDir.resolve("index.html")
            orderedPages.add(Page(entry.id, entry.title, entry.content, path))
        }

        book.entries().filter { it.categories.isEmpty() }.sortedWith(entryComparator).forEach { entry ->
            val path = bookDir.resolve("${entry.id.path}.html")
            orderedPages.add(Page(entry.id, entry.title, entry.content, path))
        }

        book.categories().sortedWith(categoryComparator).forEach { category ->
            val path = bookDir.resolve("${category.id.path}.html")
            orderedPages.add(Page(category.id, category.title, category.content, path))

            book.entriesByCategory(category)?.sortedWith(entryComparator)?.forEach { entry ->
                if (!entry.secret) {
                    val path = bookDir.resolve("${entry.id.path}.html")
                    orderedPages.add(Page(entry.id, entry.title, entry.content, path))
                }
            }
        }

        orderedPages.forEachIndexed { index, page ->
            generatePage(page.id, page.title, page.content, page.path,
                orderedPages.getOrNull(index-1)?.path, orderedPages.getOrNull(index+1)?.path)
        }

        assetDir.toFile().mkdirs()
        includeResource(Identifier.of(ColeusClient.NAME, "style.css"), "style.css")
        includeResource(
            Identifier.of(ColeusClient.NAME, "font/karla/karla-variablefont_wght.ttf"),
            "Karla-VariableFont_wght.ttf"
        )
        includeResource(Identifier.of(ColeusClient.NAME, "font/karla/ofl.txt"), "OFL.txt")
        includeResource(Identifier.of(ColeusClient.NAME, "coleus.js"), "coleus.js")

        config?.css?.forEach {
            val id = Identifier.of(it)
            val resource = MinecraftClient.getInstance().resourceManager.getResource(id).getOrNull()
            if (resource != null) {
                val outFile = assetDir.resolve(id.namespace).resolve(id.path)
                outFile.parent.toFile().mkdirs()
                resource.inputStream.use { inputStream ->
                    outFile.outputStream().use { outputStream ->
                        inputStream.transferTo(outputStream)
                    }
                }
            }
        }

        val searchEntriesFile = assetDir.resolve("searchEntries.json").toFile()
        searchEntriesFile.writeText(GsonBuilder().create().toJson(searchEntries))

        return bookDir
    }

    private fun includeResource(id: Identifier, outPath: String? = null) {
        val client = MinecraftClient.getInstance()
        val outFile = outPath?.let { assetDir.resolve(it) } ?: assetDir.resolve(id.path)

        val resource = client.resourceManager.getResource(id).getOrNull()
        if (resource != null) {
            outFile.parent.toFile().mkdirs()
            resource.inputStream.use { inputStream ->
                outFile.outputStream().use { outputStream ->
                    inputStream.transferTo(outputStream)
                }
            }
        }
    }

    private fun includeThemes() {
        val client = MinecraftClient.getInstance()

        config?.themes?.forEach { theme ->
            val resource = client.resourceManager.getResource(Identifier.of(theme.location)).getOrNull()
            if (resource != null) ThemeRegistry.put(Identifier.of(theme.id), Base16Theme.fromJsonResource(resource))
            else ColeusClient.logger.atWarn().log("could not find resource ${theme.location} for theme ${theme.id}")
        }


        val outFile = assetDir.resolve("themes.json")
        outFile.parent.toFile().mkdirs()
        val writer = outFile.writer()

        GsonBuilder().create().toJson(ThemeRegistry, writer)
        writer.close()
    }


    private fun generatePage(id: Identifier, title: String, content: String, path: Path, prevPage: Path?, nextPage: Path?) {
        val pageContext: PageContext = object : PageContext {
            override fun addSearchEntry(searchEntry: SearchEntry) = this@HtmlBookGenerator.addSearchEntry(searchEntry)

            override val pagePath: Path = path

            override val bookDir: Path = this@HtmlBookGenerator.bookDir

            override val assetsDir: Path = this@HtmlBookGenerator.assetDir
        }

        val world = MinecraftClient.getInstance().world
            ?: throw AssertionError("world must be present to generate book")

        var defaultThemeIdentifier = config?.default_theme?.let(Identifier::of)
        var defaultTheme: Base16Theme?


        if (defaultThemeIdentifier != null) {
            defaultTheme = ThemeRegistry[defaultThemeIdentifier]
            if (defaultTheme == null) {
                ColeusClient.logger.atWarn().log("could not find default theme $id for book ${book.id()}")
                defaultThemeIdentifier = Identifier.of(ColeusClient.NAME, "default")
                defaultTheme = ThemeRegistry[defaultThemeIdentifier]!!
            }
        } else {
            defaultThemeIdentifier = Identifier.of(ColeusClient.NAME, "default")
            defaultTheme = ThemeRegistry[defaultThemeIdentifier]!!
        }

        val file = path.toFile()
        file.parentFile.mkdirs()

        val bookTexture = book.texture() ?: Lavender.id("textures/gui/brown_book.png")
        val processor = MarkdownProcessor(
            { HtmlPageCompiler(pageContext) },
            BasicFormattingFeature(),
            ColorFeature(),
            LinkFeature(),
            ListFeature(),
            BlockQuoteFeature(),
            ImageFeature(),
            KeybindFeature(),
            HtmlPageBreakFeature(),
            HtmlTemplateFeature(extraParams = mapOf("book-texture" to bookTexture.toString())),
            HtmlRecipeFeature(template, LavenderBookScreenAccessor.getRecipeHandler()[book.id()]),
            HtmlOwoUIModelFeature(),
            HtmlItemStackFeature(world.registryManager),
            HtmlBlockStateFeature(),
            HtmlEntityFeature()
        )
        val outerPage = div().withId("outer-page").with(
            div().withClass("toolbar").with(
                menuButton().withId("menu-button"),
                searchButton().withId("search-button"),
                themeSelect(defaultThemeIdentifier).withId("theme-select")
            ),
        )

        val page = div().withId("page")
        val main = main(
            h1(title),
            processor.process(content),
            div().withId("mobile-page-nav").with(
                prevPage?.let { prevPageButton(path, prevPage).withId("prev-page-mobile") } ?: div(),
                nextPage?.let { nextPageButton(path, nextPage).withId("next-page-mobile") } ?: div()
            )
        )
        page.with(main)
        outerPage.with(page)

        //page switch buttons
        prevPage?.let { outerPage.with(
            prevPageButton(path, prevPage).withId("prev-page")
        )}
        nextPage?.let { outerPage.with(
            nextPageButton(path, nextPage).withId("next-page")
        )}


        val writer = file.writer()
        writer.write("<!DOCTYPE html>")
        val html = html().withStyle(defaultTheme.toCss()).with(
            head(
                loadThemeScript(),
                meta()
                    .withName("viewport")
                    .attr("content", "width=device-width,initial-scale=1"),
                link()
                    .withRel("stylesheet")
                    .withHref("${assetDir.resolve("style.css").relativeTo(path.parent)}"),
                script()
                    .withType("module")
                    .withSrc("${assetDir.resolve("coleus.js").relativeTo(path.parent)}")
                    .attr("data-assetspath", assetDir.relativeTo(path.parent).toString())
                    .attr("data-path", path.relativeTo(bookDir).toString())
                    .withId("search-script")
            ).with(extraCSS(id)),
            body().with(
                sidebar(id).withId("sidebar").attr("data-toggled", "true"),
                outerPage
            )
        )

        val searchEntry = StringBuilder()
        main.render(FlatHtml.into(searchEntry))

        addSearchEntry(
            SearchEntry(
                title,
                searchEntry.toString().replace(Regex("<[^>]*>"), " "),
                "${path.relativeTo(bookDir)}",
            )
        )

        html.render(FlatHtml.into(writer))
        writer.close()
    }

    private fun sidebar(currentPage: Identifier): DivTag {
        val uncategorizedOl = unlabeledOl()
        book.landingPage()?.let {
            uncategorizedOl.with(buildPageLink(it.id, it.title, currentPage, pagePath = "index"))
        }

        val uncategorized = book.entries().filter { it.categories.isEmpty() }
        val sidebar = div().with(buildEntryList(uncategorizedOl, uncategorized, currentPage))
        return sidebar.with(buildCategoryList(unlabeledOl(), book.categories(), currentPage))
    }

    private fun buildPageLink(pageId: Identifier, pageTitle: String, currentPage: Identifier, categoryIndex: Int? = null, index: Int? = null, pagePath: String? = null): ATag {
        val a = a()
        if (categoryIndex != null) {
            val strong = strong("${categoryIndex + 1}")
            if (index != null) strong.withText(".${index + 1} ")
            a.with(strong.withText(" "))
        }
        if (pageId == currentPage) {
            a.withId("current-page").with(strong(pageTitle))
        } else {
            a.withText(pageTitle)
        }
        return a.withHref(
            bookDir.resolve(
                "${pagePath ?: pageId.path}.html"
            ).relativeTo(bookDir.resolve(currentPage.path).parent).toString())
    }

    private fun buildCategoryList(ol: OlTag, categories: Collection<Category>, currentPage: Identifier): OlTag {
        return ol.with(categories.sortedWith(categoryComparator).mapIndexed { index, category ->
            li(
                div(
                    buildPageLink(category.id, category.title, currentPage, index),
                    book.entriesByCategory(category)?.let {
                        buildEntryList(unlabeledOl(), it, currentPage, index)
                    }
                )
            )
        })
    }

    private fun buildEntryList(ol: OlTag, entries: Collection<Entry>, currentPage: Identifier, categoryIndex: Int? = null): OlTag {
        return ol.with(
            entries.filter { !it.secret }.sortedWith(entryComparator).mapIndexed { index, entry ->
                li(
                    buildPageLink(entry.id, entry.title, currentPage, categoryIndex, index)
                )
            })
    }

    private fun extraCSS(currentPage: Identifier): List<LinkTag> {
        val css = config?.css ?: listOf()
        return css.map {
            val id = Identifier.of(it)
            link()
                .withRel("stylesheet")
                .withHref(assetDir.resolve(id.namespace).resolve(id.path).relativeTo(bookDir.resolve(currentPage.path).parent).toString())
        }
    }

    private fun unlabeledOl(): OlTag = ol().withStyle("list-style-type: none")

    private val template = object : ComponentSource {
        override fun <C : Component> template(
            model: UIModel,
            expectedComponentClass: Class<C>,
            name: String,
            parameters: MutableMap<String, String>
        ): C {
            val params = HashMap<String, String>()
            params["book-texture"] = book.texture().toString()
            params.putAll(parameters)

            return model.expandTemplate(expectedComponentClass, name, params)
        }

    }

    private fun loadThemeScript(): ScriptTag {
        return script("""
            let themeCSS = JSON.parse(localStorage.getItem("themeCSS"))
            if (themeCSS !== null) {
                for (const entry of themeCSS) {
                    document.documentElement.style.setProperty(`--${'$'}{entry[0]}`, `#${'$'}{entry[1]}`);
                }
            }
            """.trimIndent()).attr("is:inline")
    }

    private fun themeSelect(default: Identifier): SelectTag {
        return select().with(
            ThemeRegistry.toList().sortedBy { it.second.scheme }.map { (id, theme) ->
                option(theme.scheme).withValue(id.toString())
                    .withCondSelected(id == default)
                    .attr("autocomplete", "off")
            }
        )
    }

    private fun nextPageButton(path: Path, nextPage: Path): FormTag {
        return form().withId("next-page-mobile").with(
            label().with(
                input().withType("submit"),
                UnescapedText("""
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512">
                    <!--!Font Awesome Free 6.7.2 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2025 Fonticons, Inc.-->
                    <path d="M278.6 233.4c12.5 12.5 12.5 32.8 0 45.3l-160 160c-12.5 12.5-32.8 12.5-45.3 0s-12.5-32.8 0-45.3L210.7 256 73.4 118.6c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0l160 160z"/>
                </svg>
                """.trimIndent()))
        ).withAction(nextPage.relativeTo(path.parent).toString())
    }

    private fun prevPageButton(path: Path, nextPage: Path): FormTag {
        return form().withId("next-page-mobile").with(
            label().with(
                input().withType("submit"),
                UnescapedText("""
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512">
                    <!--!Font Awesome Free 6.7.2 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2025 Fonticons, Inc.-->
                    <path d="M41.4 233.4c-12.5 12.5-12.5 32.8 0 45.3l160 160c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L109.3 256 246.6 118.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0l-160 160z"/>
                </svg>
                """.trimIndent()))
        ).withAction(nextPage.relativeTo(path.parent).toString())
    }

    private fun menuButton() =
        button().attr(
            "onclick",
            """
            const btn = this;
            const sidebar = document.getElementById('sidebar');
            const toggled = sidebar.dataset.toggled === 'true';
            sidebar.dataset.toggled = toggled ? 'false' : 'true';
            """.trimIndent()
        ).with(UnescapedText(
            """
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512">
               <!--!Font Awesome Free 6.7.2 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2025 Fonticons, Inc.-->
               <path d="M0 96C0 78.3 14.3 64 32 64l384 0c17.7 0 32 14.3 32 32s-14.3 32-32 32L32 128C14.3 128 0 113.7 0 96zM0 256c0-17.7 14.3-32 32-32l384 0c17.7 0 32 14.3 32 32s-14.3 32-32 32L32 288c-17.7 0-32-14.3-32-32zM448 416c0 17.7-14.3 32-32 32L32 448c-17.7 0-32-14.3-32-32s14.3-32 32-32l384 0c17.7 0 32 14.3 32 32z"/>
            </svg>
            """.trimIndent()
        )).attr("aria-label", "toggle menu")

    private fun searchButton() =
        button(UnescapedText(
            """
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
               <!--!Font Awesome Free 6.7.2 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2025 Fonticons, Inc.-->
               <path d="M416 208c0 45.9-14.9 88.3-40 122.7L502.6 457.4c12.5 12.5 12.5 32.8 0 45.3s-32.8 12.5-45.3 0L330.7 376c-34.4 25.2-76.8 40-122.7 40C93.1 416 0 322.9 0 208S93.1 0 208 0S416 93.1 416 208zM208 352a144 144 0 1 0 0-288 144 144 0 1 0 0 288z"/>
            </svg>
            """.trimIndent()
        ))

    fun addSearchEntry(searchEntry: SearchEntry) {
        searchEntries.add(searchEntry)
    }

    private class Page(val id: Identifier, val title: String, val content: String, val path: Path)
}