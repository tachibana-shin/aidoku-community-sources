import { Hono } from "jsr:@hono/hono"

const app = new Hono()

app.get("/", async c => {
    const url = c.req.query("url")

    console.log(url)

    const html = await fetch(url).then(res => res.text())
    console.log(html)

    return c.text(html)
})

Deno.serve(app.fetch)