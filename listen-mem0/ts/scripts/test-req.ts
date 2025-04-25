const res = await fetch("http://localhost:9696/memories/search", {
  method: "POST",
  body: JSON.stringify({
    query: "what's my name",
    filters: {
      user_id: "some-user",
    },
  }),
});

console.log(res.status);

try {
  if (res.status === 200) {
    console.log(await res.json());
  } else {
    console.log(await res.text());
  }
} catch (e) {
  console.log(e);
}
