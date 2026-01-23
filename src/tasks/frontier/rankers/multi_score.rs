// TODO multi-scoring ranker

/* Consider these traits
 - Importance (depth from initial seed, how many unique inlinks)
 - Freshness (separate new from recrawl)
 - Expected yield (reward novelty)
 - Coverage (penalize websites with large url counts)
 - Cost (estimate latency from previous requests, how many errors occurred on the host prior, limit how many per host, robots.txt limits)
 - Spam (proximity to adversarial websites, content variance between pages such as templates or same titles, session ids, repeated path segments)
*/
