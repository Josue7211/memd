# memd public benchmark suite

- latest_runs: 4
- supported_targets: longmemeval, locomo, convomem, membench
- implemented_adapters: longmemeval, locomo, convomem, membench
- newest_run: membench mode=raw at 2026-04-14T02:59:53.424218691+00:00

## Target Inventory
- longmemeval: implemented
- locomo: implemented
- convomem: implemented
- membench: implemented
- implemented adapters: longmemeval, locomo, convomem, membench

## Latest Runs
| Benchmark | Version | Mode | Primary Metric | Value | Items | Dataset | Checksum | Artifacts |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ConvoMem | upstream | raw | accuracy | 1.000 | 10 | .memd/benchmarks/datasets/convomem/convomem-evidence-sample-10-per-category.json | sha256:65ec7bb06bbbc1bf169b3cb31a722f763c7c0cbce7b837c1b084215ffb9e2de9 | `.memd/benchmarks/public/convomem/latest/` |
| LoCoMo | upstream | raw | evidence_hit_rate@5 (retrieval proxy) | 0.415 | 1986 | .memd/benchmarks/datasets/locomo/locomo10.json | sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4 | `.memd/benchmarks/public/locomo/latest/` |
| LongMemEval | upstream | raw | session_recall_any@5 (retrieval proxy) | 0.828 | 500 | .memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json | sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442 | `.memd/benchmarks/public/longmemeval/latest/` |
| MemBench | upstream | raw | target_hit_rate@5 (retrieval proxy) | 0.346 | 3000 | .memd/benchmarks/datasets/membench/membench-firstagent.json | sha256:54bde8259c10ee1cfe5ff16f35a8a25ca9ad5d79e162e0b3a43034ed64115e5a | `.memd/benchmarks/public/membench/latest/` |

## Artifacts
- convomem: `.memd/benchmarks/public/convomem/latest/manifest.json`, `.memd/benchmarks/public/convomem/latest/results.json`, `.memd/benchmarks/public/convomem/latest/results.jsonl`, `.memd/benchmarks/public/convomem/latest/report.md`
- locomo: `.memd/benchmarks/public/locomo/latest/manifest.json`, `.memd/benchmarks/public/locomo/latest/results.json`, `.memd/benchmarks/public/locomo/latest/results.jsonl`, `.memd/benchmarks/public/locomo/latest/report.md`
- longmemeval: `.memd/benchmarks/public/longmemeval/latest/manifest.json`, `.memd/benchmarks/public/longmemeval/latest/results.json`, `.memd/benchmarks/public/longmemeval/latest/results.jsonl`, `.memd/benchmarks/public/longmemeval/latest/report.md`
- membench: `.memd/benchmarks/public/membench/latest/manifest.json`, `.memd/benchmarks/public/membench/latest/results.json`, `.memd/benchmarks/public/membench/latest/results.jsonl`, `.memd/benchmarks/public/membench/latest/report.md`

## Latest Run Detail: MemBench
| Item | Question | Claim | Hit | Answer | Latency ms |
| --- | --- | --- | --- | --- | --- |
| book::0::0 | What books have you recommended to me before? | raw | false | The Darwin Awards: Evolution in Action, Dude, Where's My Country? | 1 |
| book::1::0 | What books have you recommended to me before? | raw | false | Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It | 1 |
| book::2::0 | What books have you recommended to me before? | raw | false | Fat Land: How Americans Became the Fattest People in the World | 1 |
| book::3::0 | What books have you recommended to me before? | raw | false | Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks | 1 |
| book::4::0 | What books have you recommended to me before? | raw | false | Old Possum's Book of Practical Cats, Illustrated Edition, Amazing Gracie: A Dog's Tale | 1 |
| book::5::0 | What books have you recommended to me before? | raw | false | The Iron Tonic: Or, A Winter Afternoon in Lonely Valley, The Philosophy of Andy Warhol | 1 |
| book::6::0 | What books have you recommended to me before? | raw | false | ANGELA'S ASHES, Anne Frank: The Diary of a Young Girl | 1 |
| book::7::0 | What books have you recommended to me before? | raw | false | The Cases That Haunt Us | 1 |
| book::8::0 | What books have you recommended to me before? | raw | false | Many Lives, Many Masters | 1 |
| book::9::0 | What books have you recommended to me before? | raw | true | Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States, A Civil Action, One L : The Turbulent True Story of a First Year at Harvard Law School | 1 |
| book::10::0 | What books have you recommended to me before? | raw | false | Talking to Heaven: A Medium's Message of Life After Death, The Mothman Prophecies | 1 |
| book::11::0 | What books have you recommended to me before? | raw | false | A Civil Action, A Civil Action, One L : The Turbulent True Story of a First Year at Harvard Law School | 1 |
| book::12::0 | What books have you recommended to me before? | raw | false | Illusions, White Fang | 1 |
| book::13::0 | What books have you recommended to me before? | raw | false | Beowulf: A New Verse Translation | 1 |
| book::14::0 | What books have you recommended to me before? | raw | false | No Bad Dogs : The Woodhouse Way, James Herriot's Dog Stories | 1 |
| book::15::0 | What books have you recommended to me before? | raw | false | The Vagina Monologues: The V-Day Edition, Mike Nelson's Movie Megacheese, Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks | 1 |
| book::16::0 | What books have you recommended to me before? | raw | false | Tales of a Female Nomad: Living at Large in the World, In a Sunburned Country | 1 |
| book::17::0 | What books have you recommended to me before? | raw | false | Book of Virtues | 1 |
| book::18::0 | What books have you recommended to me before? | raw | false | Wildlife Preserves, Chobits (Chobits), Ghost World | 1 |
| book::19::0 | What books have you recommended to me before? | raw | false | One L : The Turbulent True Story of a First Year at Harvard Law School, A Civil Action, The Cases That Haunt Us | 1 |
| book::20::0 | What books have you recommended to me before? | raw | false | Take Care of Yourself: The Complete Illustrated Guide to Medical Self-Care, A Mind of Its Own: A Cultural History of the Penis | 1 |
| book::21::0 | What books have you recommended to me before? | raw | false | Acqua Alta, The Moonstone (Penguin Classics), 10 Lb. Penalty | 1 |
| book::22::0 | What books have you recommended to me before? | raw | false | No Bad Dogs : The Woodhouse Way | 1 |
| book::23::0 | What books have you recommended to me before? | raw | false | Team Rodent : How Disney Devours the World, Who Moved My Cheese? An Amazing Way to Deal with Change in Your Work and in Your Life | 1 |
| book::24::0 | What books have you recommended to me before? | raw | true | Left Behind: A Novel of the Earth's Last Days (Left Behind #1), Dark Water (Mira Romantic Suspense), Even Cowgirls Get the Blues | 1 |
| book::25::0 | What books have you recommended to me before? | raw | false | So You Want to Be a Wizard: The First Book in the Young Wizards Series, The Source of Magic, Prince Caspian | 1 |
| book::26::0 | What books have you recommended to me before? | raw | false | The Tipping Point: How Little Things Can Make a Big Difference, Flow: The Psychology of Optimal Experience, The Dark Side of the Light Chasers: Reclaiming Your Power, Creativity, Brilliance, and Dreams | 1 |
| book::27::0 | What books have you recommended to me before? | raw | false | The Vagina Monologues: The V-Day Edition, The Watcher's Guide 2 (Buffy the Vampire Slayer) | 1 |
| book::28::0 | What books have you recommended to me before? | raw | false | Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States, A Civil Action, The Cases That Haunt Us | 1 |
| book::29::0 | What books have you recommended to me before? | raw | true | Behind the Scenes at the Museum | 1 |
| book::30::0 | What books have you recommended to me before? | raw | false | Fat Land: How Americans Became the Fattest People in the World, Your Pregnancy: Week by Week (Your Pregnancy Series) | 1 |
| book::31::0 | What books have you recommended to me before? | raw | false | Blind Faith, Empty Promises | 1 |
| book::32::0 | What books have you recommended to me before? | raw | true | A Painted House, The Red Tent (Bestselling Backlist), To Kill a Mockingbird | 1 |
| book::33::0 | What books have you recommended to me before? | raw | true | The Moonstone (Penguin Classics), Name of the Rose, Asta's Book | 1 |
| book::34::0 | What books have you recommended to me before? | raw | true | Field of Thirteen, Die HÃ?Â¤upter meiner Lieben. | 1 |
| book::35::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Penguin Classics) | 1 |
| book::36::0 | What books have you recommended to me before? | raw | false | Chicken Soup for the College Soul : Inspiring and Humorous Stories for College Students (Chicken Soup for the Soul), Writing Down the Bones | 1 |
| book::37::0 | What books have you recommended to me before? | raw | false | The Jane Austen Book Club, The Writing Life | 1 |
| book::38::0 | What books have you recommended to me before? | raw | true | Snow Falling on Cedars, The Red Tent (Bestselling Backlist), To Kill a Mockingbird | 1 |
| book::39::0 | What books have you recommended to me before? | raw | false | Lust for Life | 1 |
| book::40::0 | What books have you recommended to me before? | raw | true | The Law, Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States | 1 |
| book::41::0 | What books have you recommended to me before? | raw | true | Selected Poems (Dover Thrift Editions), Selected Poems (Dover Thrift Edition) | 1 |
| book::42::0 | What books have you recommended to me before? | raw | true | Guns, Germs, and Steel: The Fates of Human Societies, Hiroshima | 1 |
| book::43::0 | What books have you recommended to me before? | raw | true | Hamlet (Bantam Classics), Waiting for Godot, A Streetcar Named Desire | 1 |
| book::44::0 | What books have you recommended to me before? | raw | false | Death: The High Cost of Living | 1 |
| book::45::0 | What books have you recommended to me before? | raw | false | Cinematherapy : The Girl's Guide to Movies for Every Mood | 1 |
| book::46::0 | What books have you recommended to me before? | raw | true | All Through The Night : A Suspense Story, Merrick (Vampire Chronicles), The Mists of Avalon | 1 |
| book::47::0 | What books have you recommended to me before? | raw | true | A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition | 1 |
| book::48::0 | What books have you recommended to me before? | raw | false | Brothel: Mustang Ranch and Its Women, Woman: An Intimate Geography | 1 |
| book::49::0 | What books have you recommended to me before? | raw | false | Guns, Germs, and Steel: The Fates of Human Societies, Seabiscuit | 1 |
| book::50::0 | What books have you recommended to me before? | raw | false | Angela's Ashes: A Memoir | 1 |
| book::51::0 | What books have you recommended to me before? | raw | false | Book of Virtues | 1 |
| book::52::0 | What books have you recommended to me before? | raw | true | Lies and the Lying Liars Who Tell Them: A Fair and Balanced Look at the Right, Seinlanguage, The Dilbert Principle: A Cubicle'S-Eye View of Bosses, Meetings, Management Fads & Other Workplace Afflictions | 1 |
| book::53::0 | What books have you recommended to me before? | raw | false | The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss, 8 Weeks to Optimum Health | 1 |
| book::54::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Oprah's Book Club), Liebesleben | 1 |
| book::55::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Oprah's Book Club) | 1 |
| book::56::0 | What books have you recommended to me before? | raw | false | A Walk in the Woods: Rediscovering America on the Appalachian Trail, Lies and the Lying Liars Who Tell Them: A Fair and Balanced Look at the Right | 1 |
| book::57::0 | What books have you recommended to me before? | raw | false | Seinlanguage, The Dilbert Principle: A Cubicle'S-Eye View of Bosses, Meetings, Management Fads & Other Workplace Afflictions, The Darwin Awards: Evolution in Action | 1 |
| book::58::0 | What books have you recommended to me before? | raw | true | 8 Weeks to Optimum Health | 1 |
| book::59::0 | What books have you recommended to me before? | raw | false | Hiroshima, In the Heart of the Sea: The Tragedy of the Whaleship Essex | 1 |
| book::60::0 | What books have you recommended to me before? | raw | true | Postmortem, Complicity | 1 |
| book::61::0 | What books have you recommended to me before? | raw | false | Die Gefahrten I | 1 |
| book::62::0 | What books have you recommended to me before? | raw | false | Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks | 1 |
| book::63::0 | What books have you recommended to me before? | raw | false | Culture Jam : How to Reverse America's Suicidal Consumer Binge--and Why We Must | 1 |
| book::64::0 | What books have you recommended to me before? | raw | true | The Te of Piglet, Awakening the Buddha Within : Tibetan Wisdom for the Western World | 1 |
| book::65::0 | What books have you recommended to me before? | raw | false | Many Lives, Many Masters, Wicca: A Guide for the Solitary Practitioner | 1 |
| book::66::0 | What books have you recommended to me before? | raw | true | Fraud: Essays, Ex Libris: Confessions of a Common Reader, Ex Libris : Confessions of a Common Reader | 1 |
| book::67::0 | What books have you recommended to me before? | raw | false | A Midsummer Nights Dream (Bantam Classic) | 1 |
| book::68::0 | What books have you recommended to me before? | raw | false | The Scarlet Letter: A Romance (The Penguin American Library), Lady Chatterley's Lover, Ginger Tree, Liebesleben | 1 |
| book::69::0 | What books have you recommended to me before? | raw | true | Dude, Where's My Country?, Mama Makes Up Her Mind: And Other Dangers of Southern Living, Naked | 1 |
| book::70::0 | What books have you recommended to me before? | raw | false | A Civil Action, Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States | 1 |
| book::71::0 | What books have you recommended to me before? | raw | true | Talking to Heaven: A Medium's Message of Life After Death, The Meaning Of Life, Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| book::72::0 | What books have you recommended to me before? | raw | false | Small Wonder : Essays | 1 |
| book::73::0 | What books have you recommended to me before? | raw | true | More Than Complete Hitchhiker's Guide, Restaurant At the End of the Universe | 1 |
| book::74::0 | What books have you recommended to me before? | raw | false | The Color of Water: A Black Man's Tribute to His White Mother, A Heartbreaking Work of Staggering Genius, Angela's Ashes (MMP) : A Memoir | 1 |
| book::75::0 | What books have you recommended to me before? | raw | false | Lakota Woman, There Are No Children Here: The Story of Two Boys Growing Up in the Other America, The Woman Warrior : Memoirs of a Girlhood Among Ghosts, Diet for a New America | 1 |
| book::76::0 | What books have you recommended to me before? | raw | true | Lust for Life, Wizard of Oz Postcards in Full Color (Card Books) | 1 |
| book::77::0 | What books have you recommended to me before? | raw | false | In a Sunburned Country, Mindhunter : Inside the FBI's Elite Serial Crime Unit, Empty Promises | 1 |
| book::78::0 | What books have you recommended to me before? | raw | false | The Odyssey, 100 Best-Loved Poems (Dover Thrift Editions) | 1 |
| book::79::0 | What books have you recommended to me before? | raw | true | Bibliotherapy: The Girl's Guide to Books for Every Phase of Our Lives | 1 |
| book::80::0 | What books have you recommended to me before? | raw | true | Angela's Ashes: A Memoir, The Color of Water: A Black Man's Tribute to His White Mother | 1 |
| book::81::0 | What books have you recommended to me before? | raw | false | Notes from a Small Island | 1 |
| book::82::0 | What books have you recommended to me before? | raw | false | Divorce Your Car! : Ending the Love Affair with the Automobile, A Sand County Almanac (Outdoor Essays & Reflections), When Elephants Weep: The Emotional Lives of Animals | 1 |
| book::83::0 | What books have you recommended to me before? | raw | true | Lies My Teacher Told Me : Everything Your American History Textbook Got Wrong, Midnight in the Garden of Good and Evil: A Savannah Story | 1 |
| book::84::0 | What books have you recommended to me before? | raw | false | The Doll's House (Sandman, Book 2), Chobits (Chobits), Attack Of The Deranged Mutant Killer Snow Goons | 1 |
| book::85::0 | What books have you recommended to me before? | raw | false | Watchmen | 1 |
| book::86::0 | What books have you recommended to me before? | raw | true | Angela's Ashes (MMP) : A Memoir, Angela's Ashes: A Memoir | 1 |
| book::87::0 | What books have you recommended to me before? | raw | false | Woman: An Intimate Geography, Lakota Woman | 1 |
| book::88::0 | What books have you recommended to me before? | raw | false | Empty Promises, Small Sacrifices: A True Story of Passion and Murder | 1 |
| book::89::0 | What books have you recommended to me before? | raw | false | Die Gefahrten I, The Fellowship of the Ring (The Lord of the Rings, Part 1), El Senor De Los Anillos: El Retorno Del Rey (Tolkien, J. R. R. Lord of the Rings. 3.) | 1 |
| book::90::0 | What books have you recommended to me before? | raw | true | Seabiscuit: An American Legend, A Year in Provence, In the Heart of the Sea: The Tragedy of the Whaleship Essex | 1 |
| book::91::0 | What books have you recommended to me before? | raw | true | So You Want to Be a Wizard: The First Book in the Young Wizards Series, The Magician's Nephew | 1 |
| book::92::0 | What books have you recommended to me before? | raw | false | The Tao of Pooh | 1 |
| book::93::0 | What books have you recommended to me before? | raw | false | The Curious Sofa: A Pornographic Work by Ogdred Weary | 1 |
| book::94::0 | What books have you recommended to me before? | raw | true | Ginger Tree, Lady Chatterley's Lover, Anna Karenina (Oprah's Book Club), Love in the Time of Cholera (Penguin Great Books of the 20th Century) | 1 |
| book::95::0 | What books have you recommended to me before? | raw | false | The Prince, The O'Reilly Factor: The Good, the Bad, and the Completely Ridiculous in American Life | 1 |
| book::96::0 | What books have you recommended to me before? | raw | false | In the Heart of the Sea: The Tragedy of the Whaleship Essex, Hiroshima, Seabiscuit: An American Legend | 1 |
| book::97::0 | What books have you recommended to me before? | raw | false | Mindhunter : Inside the FBI's Elite Serial Crime Unit, Bitter Harvest, EVERYTHING SHE EVER WANTED | 1 |
| book::98::0 | What books have you recommended to me before? | raw | true | So Long and Thanks for all the Fish, More Than Complete Hitchhiker's Guide, Restaurant At the End of the Universe | 1 |
| book::99::0 | What books have you recommended to me before? | raw | false | Their eyes were watching God: A novel, The Screwtape Letters | 1 |
| book::100::0 | What books have you recommended to me before? | raw | false | Orfe, Go Ask Alice (Avon/Flare Book) | 1 |
| book::101::0 | What books have you recommended to me before? | raw | false | Walden and Other Writings | 1 |
| book::102::0 | What books have you recommended to me before? | raw | false | The Magician's Nephew | 1 |
| book::103::0 | What books have you recommended to me before? | raw | true | MY SWEET AUDRINA, SHIPPING NEWS | 1 |
| book::104::0 | What books have you recommended to me before? | raw | true | Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It, The Man Who Mistook His Wife for a Hat: And Other Clinical Tales, A Mind of Its Own: A Cultural History of the Penis | 1 |
| book::105::0 | What books have you recommended to me before? | raw | false | Beowulf: A New Verse Translation, Sonnets from the Portuguese and Other Poems (Dover Thrift Editions) | 1 |
| book::106::0 | What books have you recommended to me before? | raw | false | The Importance of Being Earnest (Dover Thrift Editions), Romeo and Juliet (Bantam Classic) | 1 |
| book::107::0 | What books have you recommended to me before? | raw | false | The Tao of Pooh | 1 |
| book::108::0 | What books have you recommended to me before? | raw | true | Good Faeries Bad Faeries, The Philosophy of Andy Warhol | 1 |
| book::109::0 | What books have you recommended to me before? | raw | false | Brothel: Mustang Ranch and Its Women | 1 |
| book::110::0 | What books have you recommended to me before? | raw | true | The Man Who Listens to Horses | 1 |
| book::111::0 | What books have you recommended to me before? | raw | true | One Hundred Ways for a Cat to Train Its Human | 1 |
| book::112::0 | What books have you recommended to me before? | raw | false | Parliament of Whores: A Lone Humorist Attempts to Explain the Entire U.S. Government | 1 |
| book::113::0 | What books have you recommended to me before? | raw | false | The Sweet Potato Queens' Book of Love, If the Buddha Dated: A Handbook for Finding Love on a Spiritual Path | 1 |
| book::114::0 | What books have you recommended to me before? | raw | false | Stiff: The Curious Lives of Human Cadavers, The Elements of Style, Fourth Edition, Wild Mind: Living the Writer's Life | 1 |
| book::115::0 | What books have you recommended to me before? | raw | false | Foundations Edge, Strata | 1 |
| book::116::0 | What books have you recommended to me before? | raw | true | The Plague, It Was on Fire When I Lay Down on It, Small Wonder: Essays, Ex Libris : Confessions of a Common Reader | 1 |
| book::117::0 | What books have you recommended to me before? | raw | true | Chicken Soup for the Soul (Chicken Soup for the Soul) | 1 |
| book::118::0 | What books have you recommended to me before? | raw | false | All Through The Night : A Suspense Story | 1 |
| book::119::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Penguin Classics), The Jane Austen Book Club | 1 |
| book::120::0 | What books have you recommended to me before? | raw | false | Fix-It and Forget-It Cookbook: Feasting with Your Slow Cooker, A Kitchen Witch's Cookbook | 1 |
| book::121::0 | What books have you recommended to me before? | raw | false | HEARTBURN, Das Hotel New Hampshire | 1 |
| book::122::0 | What books have you recommended to me before? | raw | false | Eats, Shoots and Leaves: The Zero Tolerance Approach to Punctuation | 1 |
| book::123::0 | What books have you recommended to me before? | raw | false | Wizard of Oz Postcards in Full Color (Card Books), Why Cats Paint: A Theory of Feline Aesthetics, The Iron Tonic: Or, A Winter Afternoon in Lonely Valley | 1 |
| book::124::0 | What books have you recommended to me before? | raw | false | Harry Potter and the Goblet of Fire (Book 4), Harry Potter and the Sorcerer's Stone (Book 1) | 1 |
| book::125::0 | What books have you recommended to me before? | raw | false | The Fellowship of the Ring, El Senor De Los Anillos: El Retorno Del Rey (Tolkien, J. R. R. Lord of the Rings. 3.), El Senor De Los Anillos: LA Comunidad Del Anillo (Lord of the Rings (Spanish)) | 1 |
| book::126::0 | What books have you recommended to me before? | raw | false | Woman: An Intimate Geography, Culture Jam : How to Reverse America's Suicidal Consumer Binge--and Why We Must | 1 |
| book::127::0 | What books have you recommended to me before? | raw | true | What to Expect When You're Expecting (Revised Edition), 8 Weeks to Optimum Health, Make the Connection: Ten Steps to a Better Body and a Better Life | 1 |
| book::128::0 | What books have you recommended to me before? | raw | true | Diet for a New America, American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library), The Woman Warrior : Memoirs of a Girlhood Among Ghosts | 1 |
| book::129::0 | What books have you recommended to me before? | raw | false | A Walk in the Woods: Rediscovering America on the Appalachian Trail, Dude, Where's My Country? | 1 |
| book::130::0 | What books have you recommended to me before? | raw | false | Stupid White Men : ...And Other Sorry Excuses for the State of the Nation!, 9-11 | 1 |
| book::131::0 | What books have you recommended to me before? | raw | true | The Perfect Storm : A True Story of Men Against the Sea, The Snow Leopard (Penguin Nature Classics) | 1 |
| book::132::0 | What books have you recommended to me before? | raw | false | Wicca: A Guide for the Solitary Practitioner, Many Lives, Many Masters, The Meaning Of Life | 1 |
| book::133::0 | What books have you recommended to me before? | raw | true | Anger: Wisdom for Cooling the Flames, What Should I Do with My Life?, Awakening the Buddha Within : Tibetan Wisdom for the Western World | 1 |
| book::134::0 | What books have you recommended to me before? | raw | false | Creative Companion: How to Free Your Creative Spirit | 1 |
| book::135::0 | What books have you recommended to me before? | raw | false | The Golden Compass (His Dark Materials, Book 1), Harry Potter and the Sorcerer's Stone (Book 1) | 1 |
| book::136::0 | What books have you recommended to me before? | raw | false | Chicken Soup for the Pet Lover's Soul (Chicken Soup for the Soul) | 1 |
| book::137::0 | What books have you recommended to me before? | raw | false | White Fang, The Street Lawyer | 1 |
| book::138::0 | What books have you recommended to me before? | raw | false | Hamlet (Bantam Classics) | 1 |
| book::139::0 | What books have you recommended to me before? | raw | true | Coma (Signet Books), The Clan of the Cave Bear : a novel, Angels and Demons | 1 |
| book::140::0 | What books have you recommended to me before? | raw | true | The Red Tent (Bestselling Backlist), The Secret Life of Bees, Good in Bed | 1 |
| book::141::0 | What books have you recommended to me before? | raw | true | The Meaning Of Life | 1 |
| book::142::0 | What books have you recommended to me before? | raw | false | Good in Bed, To Kill a Mockingbird | 1 |
| book::143::0 | What books have you recommended to me before? | raw | false | The Elements of Style, Fourth Edition, Wild Mind: Living the Writer's Life | 1 |
| book::144::0 | What books have you recommended to me before? | raw | false | 100 Selected Poems by E. E. Cummings | 1 |
| book::145::0 | What books have you recommended to me before? | raw | false | Field of Thirteen, Debout les morts | 1 |
| book::146::0 | What books have you recommended to me before? | raw | false | Shipping News | 1 |
| book::147::0 | What books have you recommended to me before? | raw | false | The Prince, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| book::148::0 | What books have you recommended to me before? | raw | false | The Scarlet Letter: A Romance (The Penguin American Library), Liebesleben | 1 |
| book::149::0 | What books have you recommended to me before? | raw | true | The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss, Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements, What to Expect When You're Expecting (Revised Edition) | 1 |
| book::150::0 | What books have you recommended to me before? | raw | false | The Te of Piglet, The Te of Piglet | 1 |
| book::151::0 | What books have you recommended to me before? | raw | true | The Grey King (The Dark is Rising Sequence) | 1 |
| book::152::0 | What books have you recommended to me before? | raw | false | Stupid White Men ...and Other Sorry Excuses for the State of the Nation!, Bush at War, A Royal Duty, The Prince | 1 |
| book::153::0 | What books have you recommended to me before? | raw | false | Odd Girl Out: The Hidden Culture of Aggression in Girls, Man's Search for Meaning: An Introduction to Logotherapy | 1 |
| book::154::0 | What books have you recommended to me before? | raw | true | Hiroshima, Seabiscuit: An American Legend, A Year in Provence | 1 |
| book::155::0 | What books have you recommended to me before? | raw | false | Brothel: Mustang Ranch and Its Women | 1 |
| book::156::0 | What books have you recommended to me before? | raw | false | The Demon-Haunted World: Science As a Candle in the Dark, A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition | 1 |
| book::157::0 | What books have you recommended to me before? | raw | false | Chocolate: The Consuming Passion, In the Kitchen With Rosie: Oprah's Favorite Recipes | 1 |
| book::158::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Penguin Classics), Lonely Planet Unpacked | 1 |
| book::159::0 | What books have you recommended to me before? | raw | false | The Meaning Of Life, SEAT OF THE SOUL | 1 |
| book::160::0 | What books have you recommended to me before? | raw | true | Hop on Pop (I Can Read It All by Myself Beginner Books), Go Ask Alice (Avon/Flare Book) | 1 |
| book::161::0 | What books have you recommended to me before? | raw | false | Debout les morts | 1 |
| book::162::0 | What books have you recommended to me before? | raw | false | Rosencrantz & Guildenstern Are Dead, The Importance of Being Earnest (Dover Thrift Editions) | 1 |
| book::163::0 | What books have you recommended to me before? | raw | false | Liebesleben, The Scarlet Letter: A Romance (The Penguin American Library) | 1 |
| book::164::0 | What books have you recommended to me before? | raw | false | The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them | 1 |
| book::165::0 | What books have you recommended to me before? | raw | true | Angela's Ashes (MMP) : A Memoir, Lucky Man: A Memoir | 1 |
| book::166::0 | What books have you recommended to me before? | raw | false | A Natural History of the Senses | 1 |
| book::167::0 | What books have you recommended to me before? | raw | false | 9-11, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| book::168::0 | What books have you recommended to me before? | raw | false | Writing Down the Bones, The Four Agreements: A Practical Guide to Personal Freedom | 1 |
| book::169::0 | What books have you recommended to me before? | raw | false | A Rage To Kill and Other True Cases : Anne Rule's Crime Files, Vol. 6 (Ann Rule's Crime Files), Bitter Harvest | 1 |
| book::170::0 | What books have you recommended to me before? | raw | false | A Midsummer Nights Dream (Bantam Classic) | 1 |
| book::171::0 | What books have you recommended to me before? | raw | true | The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child, Uncle Shelby's ABZ Book: A Primer for Adults Only | 1 |
| book::172::0 | What books have you recommended to me before? | raw | false | Seinlanguage, Dude, Where's My Country? | 1 |
| book::173::0 | What books have you recommended to me before? | raw | false | GefÃ?Â¤hrliche Geliebte. | 1 |
| book::174::0 | What books have you recommended to me before? | raw | true | Mars and Venus on a Date : A Guide to Navigating the 5 Stages of Dating to Create a Loving and Lasting Relationship, What to Expect the First Year | 1 |
| book::175::0 | What books have you recommended to me before? | raw | false | Empire Strikes Back Wars, The Hobbit | 1 |
| book::176::0 | What books have you recommended to me before? | raw | false | Restaurant At the End of the Universe, So Long and Thanks for all the Fish | 1 |
| book::177::0 | What books have you recommended to me before? | raw | false | New Vegetarian: Bold and Beautiful Recipes for Every Occasion | 1 |
| book::178::0 | What books have you recommended to me before? | raw | false | A Kitchen Witch's Cookbook, A Cook's Tour | 1 |
| book::179::0 | What books have you recommended to me before? | raw | false | Romeo and Juliet (Dover Thrift Editions), The Importance of Being Earnest (Dover Thrift Editions) | 1 |
| book::180::0 | What books have you recommended to me before? | raw | true | Harry Potter and the Chamber of Secrets (Book 2), Harry Potter and the Sorcerer's Stone (Harry Potter (Paperback)), A Wrinkle In Time | 1 |
| book::181::0 | What books have you recommended to me before? | raw | false | Ciao, America: An Italian Discovers the U.S, In a Sunburned Country | 1 |
| book::182::0 | What books have you recommended to me before? | raw | false | Small Wonder: Essays | 1 |
| book::183::0 | What books have you recommended to me before? | raw | true | To Ride a Silver Broomstick: New Generation Witchcraft, SEAT OF THE SOUL | 1 |
| book::184::0 | What books have you recommended to me before? | raw | false | Seabiscuit: An American Legend, Seabiscuit | 1 |
| book::185::0 | What books have you recommended to me before? | raw | false | Dr. Atkins' New Diet Revolution, Fat Land: How Americans Became the Fattest People in the World | 1 |
| book::186::0 | What books have you recommended to me before? | raw | false | The Fellowship of the Ring (The Lord of the Rings, Part 1), The Fellowship of the Ring, El Senor De Los Anillos: El Retorno Del Rey (Tolkien, J. R. R. Lord of the Rings. 3.) | 1 |
| book::187::0 | What books have you recommended to me before? | raw | false | The Scarlet Letter: A Romance (The Penguin American Library), Anna Karenina (Oprah's Book Club), Ginger Tree | 1 |
| book::188::0 | What books have you recommended to me before? | raw | false | Route 66 Postcards: Greetings from the Mother Road, Ciao, America: An Italian Discovers the U.S | 1 |
| book::189::0 | What books have you recommended to me before? | raw | false | Cosmos, The Universe in a Nutshell | 1 |
| book::190::0 | What books have you recommended to me before? | raw | false | Cinematherapy : The Girl's Guide to Movies for Every Mood, The Simpsons and Philosophy: The D'oh! of Homer | 1 |
| book::191::0 | What books have you recommended to me before? | raw | false | Body for Life: 12 Weeks to Mental and Physical Strength, Fat Land: How Americans Became the Fattest People in the World | 1 |
| book::192::0 | What books have you recommended to me before? | raw | false | Chobits Vol.1, Watchmen | 1 |
| book::193::0 | What books have you recommended to me before? | raw | true | The Man Who Listens to Horses, The Snow Leopard (Penguin Nature Classics), A Sand County Almanac (Outdoor Essays & Reflections) | 1 |
| book::194::0 | What books have you recommended to me before? | raw | true | The Dark Side of the Light Chasers: Reclaiming Your Power, Creativity, Brilliance, and Dreams, You Just Don't Understand | 1 |
| book::195::0 | What books have you recommended to me before? | raw | false | The Moonstone (Penguin Classics), 10 Lb. Penalty | 1 |
| book::196::0 | What books have you recommended to me before? | raw | true | EVERYTHING SHE EVER WANTED, Catch Me If You Can: The True Story of a Real Fake, Small Sacrifices: A True Story of Passion and Murder | 1 |
| book::197::0 | What books have you recommended to me before? | raw | false | Death: The High Cost of Living | 1 |
| book::198::0 | What books have you recommended to me before? | raw | false | Wizard of Oz Postcards in Full Color (Card Books), Lust for Life | 1 |
| book::199::0 | What books have you recommended to me before? | raw | false | The Last Battle (The Chronicles of Narnia Book 7), El Principito, The Silver Chair | 1 |
| book::200::0 | What books have you recommended to me before? | raw | false | Body for Life: 12 Weeks to Mental and Physical Strength, Dr. Atkins' New Diet Revolution | 1 |
| book::201::0 | What books have you recommended to me before? | raw | true | The Purpose-Driven Life: What on Earth Am I Here For?, The Prayer of Jabez: Breaking Through to the Blessed Life, Nine Parts of Desire: The Hidden World of Islamic Women | 1 |
| book::202::0 | What books have you recommended to me before? | raw | true | The Hobbit, The Silence of the Lambs, Tommo & Hawk | 1 |
| book::203::0 | What books have you recommended to me before? | raw | false | Mansfield Park (Penguin Classics), Red Dwarf | 1 |
| book::204::0 | What books have you recommended to me before? | raw | false | The Blue Day Book, A 5th Portion of Chicken Soup for the Soul : 101 Stories to Open the Heart and Rekindle the Spirit | 1 |
| book::205::0 | What books have you recommended to me before? | raw | true | Mike Nelson's Movie Megacheese, Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks, Cinematherapy : The Girl's Guide to Movies for Every Mood | 1 |
| book::206::0 | What books have you recommended to me before? | raw | false | Downsize This! Random Threats from an Unarmed American, We're Right, They're Wrong: A Handbook for Spirited Progressives | 1 |
| book::207::0 | What books have you recommended to me before? | raw | false | The Moonstone (Penguin Classics) | 1 |
| book::208::0 | What books have you recommended to me before? | raw | true | Ghost World | 1 |
| book::209::0 | What books have you recommended to me before? | raw | false | Coma (Signet Books) | 1 |
| book::210::0 | What books have you recommended to me before? | raw | false | Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It, Take Care of Yourself: The Complete Illustrated Guide to Medical Self-Care, A Mind of Its Own: A Cultural History of the Penis, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| book::211::0 | What books have you recommended to me before? | raw | false | The Sweet Potato Queens' Book of Love | 1 |
| book::212::0 | What books have you recommended to me before? | raw | false | A Heartbreaking Work of Staggering Genius | 1 |
| book::213::0 | What books have you recommended to me before? | raw | false | Nine Parts of Desire: The Hidden World of Islamic Women | 1 |
| book::214::0 | What books have you recommended to me before? | raw | false | The Meaning Of Life, Embraced by the Light | 1 |
| book::215::0 | What books have you recommended to me before? | raw | false | Prince and the Pauper Walt Disney | 1 |
| book::216::0 | What books have you recommended to me before? | raw | false | Go Ask Alice (Avon/Flare Book) | 1 |
| book::217::0 | What books have you recommended to me before? | raw | true | Snow Falling on Cedars, The Five People You Meet in Heaven | 1 |
| book::218::0 | What books have you recommended to me before? | raw | false | Sense and Sensibility (World's Classics) | 1 |
| book::219::0 | What books have you recommended to me before? | raw | false | The Writing Life | 1 |
| book::220::0 | What books have you recommended to me before? | raw | false | The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child, Savage Inequalities: Children in America's Schools | 1 |
| book::221::0 | What books have you recommended to me before? | raw | false | The Bad Beginning (A Series of Unfortunate Events, Book 1) | 1 |
| book::222::0 | What books have you recommended to me before? | raw | true | A Royal Duty, The O'Reilly Factor: The Good, the Bad, and the Completely Ridiculous in American Life | 1 |
| book::223::0 | What books have you recommended to me before? | raw | true | The Te of Piglet, The Tao of Pooh | 1 |
| book::224::0 | What books have you recommended to me before? | raw | true | Uncle Shelby's ABZ Book: A Primer for Adults Only, The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them, Savage Inequalities: Children in America's Schools | 1 |
| book::225::0 | What books have you recommended to me before? | raw | false | Mansfield Park (Penguin Classics) | 1 |
| book::226::0 | What books have you recommended to me before? | raw | false | Snow Falling on Cedars, A Painted House | 1 |
| book::227::0 | What books have you recommended to me before? | raw | true | Ginger Tree | 1 |
| book::228::0 | What books have you recommended to me before? | raw | false | Bird by Bird: Some Instructions on Writing and Life, Eats, Shoots and Leaves: The Zero Tolerance Approach to Punctuation | 1 |
| book::229::0 | What books have you recommended to me before? | raw | false | Fraud: Essays, Walden and Other Writings, Small Wonder: Essays, Small Wonder : Essays | 1 |
| book::230::0 | What books have you recommended to me before? | raw | false | Cats and Their Women | 1 |
| book::231::0 | What books have you recommended to me before? | raw | true | Coma (Signet Books), El Guardian Entre El Centeno, Angels and Demons | 1 |
| book::232::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Penguin Classics), In the Name of Love : Ann Rule's Crime Files Volume 4 (Ann Rule's Crime Files) | 1 |
| book::233::0 | What books have you recommended to me before? | raw | true | Man's Search for Meaning: An Introduction to Logotherapy, The Anatomy of Motive : The FBI's Legendary Mindhunter Explores the Key to Understanding and Catching Violent Criminals, The Psychologist's Book of Self-Tests: 25 Love, Sex, Intelligence, Career, and Personality Tests Developed by Professionals to Reveal the Real You | 1 |
| book::234::0 | What books have you recommended to me before? | raw | true | Alive : The Story of the Andes Survivors (Avon Nonfiction), McCarthy's Bar: A Journey of Discovery In Ireland | 1 |
| book::235::0 | What books have you recommended to me before? | raw | false | This Present Darkness | 1 |
| book::236::0 | What books have you recommended to me before? | raw | false | Scientific Progress Goes 'Boink':  A Calvin and Hobbes Collection, The Doll's House (Sandman, Book 2) | 1 |
| book::237::0 | What books have you recommended to me before? | raw | false | The Universe in a Nutshell, The Demon-Haunted World: Science As a Candle in the Dark | 1 |
| book::238::0 | What books have you recommended to me before? | raw | true | Asta's Book, 10 Lb. Penalty | 1 |
| book::239::0 | What books have you recommended to me before? | raw | false | Uncle Shelby's ABZ Book: A Primer for Adults Only | 1 |
| book::240::0 | What books have you recommended to me before? | raw | true | One L : The Turbulent True Story of a First Year at Harvard Law School | 1 |
| book::241::0 | What books have you recommended to me before? | raw | false | Angela's Ashes: A Memoir, Lucky Man: A Memoir | 1 |
| book::242::0 | What books have you recommended to me before? | raw | true | Another Roadside Attraction, A Dangerous Fortune | 1 |
| book::243::0 | What books have you recommended to me before? | raw | false | Who Moved My Cheese? An Amazing Way to Deal with Change in Your Work and in Your Life | 1 |
| book::244::0 | What books have you recommended to me before? | raw | false | Empire Strikes Back Wars, The Silence of the Lambs | 1 |
| book::245::0 | What books have you recommended to me before? | raw | false | Behind the Scenes at the Museum, Hard Times for These Times (English Library), SHIPPING NEWS | 1 |
| book::246::0 | What books have you recommended to me before? | raw | true | The Law, A Civil Action | 1 |
| book::247::0 | What books have you recommended to me before? | raw | false | The Silver Chair, The Source of Magic | 1 |
| book::248::0 | What books have you recommended to me before? | raw | false | Tommo & Hawk | 1 |
| book::249::0 | What books have you recommended to me before? | raw | false | Old Possum's Book of Practical Cats, Illustrated Edition, ALL MY PATIENTS ARE UNDER THE BED | 1 |
| book::250::0 | What books have you recommended to me before? | raw | false | The Demon-Haunted World: Science As a Candle in the Dark, Genome | 1 |
| book::251::0 | What books have you recommended to me before? | raw | false | HEARTBURN, Behind the Scenes at the Museum | 1 |
| book::252::0 | What books have you recommended to me before? | raw | false | Ginger Tree, Anna Karenina (Oprah's Book Club) | 1 |
| book::253::0 | What books have you recommended to me before? | raw | false | The Law, A Civil Action | 1 |
| book::254::0 | What books have you recommended to me before? | raw | true | A Painted House, The Five People You Meet in Heaven, The Red Tent (Bestselling Backlist) | 1 |
| book::255::0 | What books have you recommended to me before? | raw | true | The Magician's Nephew, The Source of Magic, So You Want to Be a Wizard: The First Book in the Young Wizards Series | 1 |
| book::256::0 | What books have you recommended to me before? | raw | false | Talking to Heaven: A Medium's Message of Life After Death, To Ride a Silver Broomstick: New Generation Witchcraft | 1 |
| book::257::0 | What books have you recommended to me before? | raw | false | The Blue Day Book | 1 |
| book::258::0 | What books have you recommended to me before? | raw | false | Lies and the Lying Liars Who Tell Them: A Fair and Balanced Look at the Right, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| book::259::0 | What books have you recommended to me before? | raw | false | In a Sunburned Country | 1 |
| book::260::0 | What books have you recommended to me before? | raw | false | Chobits Vol.1 | 1 |
| book::261::0 | What books have you recommended to me before? | raw | false | Brothel: Mustang Ranch and Its Women, The Tipping Point: How Little Things Can Make a Big Difference | 1 |
| book::262::0 | What books have you recommended to me before? | raw | false | Chicken Soup for the Christian Soul (Chicken Soup for the Soul Series (Paper)), Plain and Simple : A Journey to the Amish (Ohio) | 1 |
| book::263::0 | What books have you recommended to me before? | raw | true | Even Cowgirls Get the Blues, The Mists of Avalon | 1 |
| book::264::0 | What books have you recommended to me before? | raw | false | The Girlfriends' Guide to Pregnancy | 1 |
| book::265::0 | What books have you recommended to me before? | raw | true | Don't Sweat the Small Stuff and It's All Small Stuff : Simple Ways to Keep the Little Things from Taking Over Your Life (Don't Sweat the Small Stuff Series), Life Strategies: Doing What Works, Doing What Matters | 1 |
| book::266::0 | What books have you recommended to me before? | raw | false | Illusions | 1 |
| book::267::0 | What books have you recommended to me before? | raw | false | New Vegetarian: Bold and Beautiful Recipes for Every Occasion, A Kitchen Witch's Cookbook, Chocolate: The Consuming Passion | 1 |
| book::268::0 | What books have you recommended to me before? | raw | false | Make the Connection: Ten Steps to a Better Body and a Better Life | 1 |
| book::269::0 | What books have you recommended to me before? | raw | true | McCarthy's Bar: A Journey of Discovery In Ireland, Tales of a Female Nomad: Living at Large in the World | 1 |
| book::270::0 | What books have you recommended to me before? | raw | false | Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks, The Vagina Monologues: The V-Day Edition | 1 |
| book::271::0 | What books have you recommended to me before? | raw | false | Women Who Run with the Wolves, Lonely Planet Unpacked | 1 |
| book::272::0 | What books have you recommended to me before? | raw | true | Culture Jam : How to Reverse America's Suicidal Consumer Binge--and Why We Must | 1 |
| book::273::0 | What books have you recommended to me before? | raw | true | Bird by Bird: Some Instructions on Writing and Life, Amusing Ourselves to Death: Public Discourse in the Age of Show Business | 1 |
| book::274::0 | What books have you recommended to me before? | raw | true | Do What You Love, The Money Will Follow : Discovering Your Right Livelihood, SEVEN HABITS OF HIGHLY EFFECTIVE PEOPLE : Powerful Lessons in Personal Change | 1 |
| book::275::0 | What books have you recommended to me before? | raw | true | The Golden Compass (His Dark Materials, Book 1) | 1 |
| book::276::0 | What books have you recommended to me before? | raw | false | The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss, Fat Land: How Americans Became the Fattest People in the World | 1 |
| book::277::0 | What books have you recommended to me before? | raw | false | Make the Connection: Ten Steps to a Better Body and a Better Life, Body for Life: 12 Weeks to Mental and Physical Strength, 8 Weeks to Optimum Health | 1 |
| book::278::0 | What books have you recommended to me before? | raw | false | The Mists of Avalon | 1 |
| book::279::0 | What books have you recommended to me before? | raw | false | An Anthropologist on Mars: Seven Paradoxical Tales, The Man Who Mistook His Wife for a Hat: And Other Clinical Tales | 1 |
| book::280::0 | What books have you recommended to me before? | raw | false | ALL MY PATIENTS ARE UNDER THE BED | 1 |
| book::281::0 | What books have you recommended to me before? | raw | false | The Tao of Pooh, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| book::282::0 | What books have you recommended to me before? | raw | true | Amusing Ourselves to Death: Public Discourse in the Age of Show Business, Eats, Shoots and Leaves: The Zero Tolerance Approach to Punctuation | 1 |
| book::283::0 | What books have you recommended to me before? | raw | false | The original Hitchhiker radio scripts, So Long and Thanks for all the Fish | 1 |
| book::284::0 | What books have you recommended to me before? | raw | false | Selected Poems (Dover Thrift Edition) | 1 |
| book::285::0 | What books have you recommended to me before? | raw | false | The Man Who Listens to Horses, The Snow Leopard (Penguin Nature Classics) | 1 |
| book::286::0 | What books have you recommended to me before? | raw | true | The Magician's Nephew, Prince Caspian | 1 |
| book::287::0 | What books have you recommended to me before? | raw | false | Selected Poems (Dover Thrift Editions), 100 Best-Loved Poems (Dover Thrift Editions), Sonnets from the Portuguese and Other Poems (Dover Thrift Editions) | 1 |
| book::288::0 | What books have you recommended to me before? | raw | false | A Sand County Almanac (Outdoor Essays & Reflections), The Perfect Storm : A True Story of Men Against the Sea, The Snow Leopard (Penguin Nature Classics) | 1 |
| book::289::0 | What books have you recommended to me before? | raw | false | Plain and Simple : A Journey to the Amish (Ohio), The Prayer of Jabez: Breaking Through to the Blessed Life, Chicken Soup for the Christian Soul (Chicken Soup for the Soul Series (Paper)) | 1 |
| book::290::0 | What books have you recommended to me before? | raw | false | If the Buddha Dated: A Handbook for Finding Love on a Spiritual Path | 1 |
| book::291::0 | What books have you recommended to me before? | raw | false | Another Roadside Attraction | 1 |
| book::292::0 | What books have you recommended to me before? | raw | false | Parliament of Whores: A Lone Humorist Attempts to Explain the Entire U.S. Government | 1 |
| book::293::0 | What books have you recommended to me before? | raw | false | Midnight in the Garden of Good and Evil: A Savannah Story | 1 |
| book::294::0 | What books have you recommended to me before? | raw | false | High Tide in Tucson : Essays from Now or Never, The Plague | 1 |
| book::295::0 | What books have you recommended to me before? | raw | true | A Year in Provence | 1 |
| book::296::0 | What books have you recommended to me before? | raw | false | 10 Lb. Penalty, Acqua Alta | 1 |
| book::297::0 | What books have you recommended to me before? | raw | false | DEAD BY SUNSET : DEAD BY SUNSET | 1 |
| book::298::0 | What books have you recommended to me before? | raw | false | Sense and Sensibility, Kitchen | 1 |
| book::299::0 | What books have you recommended to me before? | raw | true | Enigma., Strata | 1 |
| book::300::0 | What books have you recommended to me before? | raw | false | Red Dwarf, Strata, The Greatest Show Off Earth, GefÃ?Â¤hrliche Geliebte. | 1 |
| book::301::0 | What books have you recommended to me before? | raw | true | Book Lust: Recommended Reading for Every Mood, Moment, and Reason, The Jane Austen Book Club | 1 |
| book::302::0 | What books have you recommended to me before? | raw | false | Savage Inequalities: Children in America's Schools, The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child | 1 |
| book::303::0 | What books have you recommended to me before? | raw | false | The Woman Warrior : Memoirs of a Girlhood Among Ghosts | 1 |
| book::304::0 | What books have you recommended to me before? | raw | false | Harry Potter and the Sorcerer's Stone (Harry Potter (Paperback)), The Golden Compass (His Dark Materials, Book 1), The Bad Beginning (A Series of Unfortunate Events, Book 1), Harry Potter and the Chamber of Secrets (Book 2) | 1 |
| book::305::0 | What books have you recommended to me before? | raw | false | Different Seasons | 1 |
| book::306::0 | What books have you recommended to me before? | raw | true | Team Rodent : How Disney Devours the World, Nickel and Dimed: On (Not) Getting By in America, Fish! A Remarkable Way to Boost Morale and Improve Results | 1 |
| book::307::0 | What books have you recommended to me before? | raw | false | Genome, My Family and Other Animals. | 1 |
| book::308::0 | What books have you recommended to me before? | raw | false | Notes from a Small Island, Route 66 Postcards: Greetings from the Mother Road, Neither Here nor There: Travels in Europe, Tales of a Female Nomad: Living at Large in the World | 1 |
| book::309::0 | What books have you recommended to me before? | raw | true | The Anatomy of Motive : The FBI's Legendary Mindhunter Explores the Key to Understanding and Catching Violent Criminals, Flow: The Psychology of Optimal Experience, The Tipping Point: How Little Things Can Make a Big Difference | 1 |
| book::310::0 | What books have you recommended to me before? | raw | false | Diet for a Small Planet (20th Anniversary Edition), Chocolate: The Consuming Passion | 1 |
| book::311::0 | What books have you recommended to me before? | raw | false | Uncle Shelby's ABZ Book: A Primer for Adults Only, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them, Savage Inequalities: Children in America's Schools | 1 |
| book::312::0 | What books have you recommended to me before? | raw | false | The Perfect Storm : A True Story of Men Against the Sea, The Man Who Listens to Horses | 1 |
| book::313::0 | What books have you recommended to me before? | raw | true | The Color of Water: A Black Man's Tribute to His White Mother, A Heartbreaking Work of Staggering Genius | 1 |
| book::314::0 | What books have you recommended to me before? | raw | false | Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements, Dr. Atkins' New Diet Revolution | 1 |
| book::315::0 | What books have you recommended to me before? | raw | true | El Principito, So You Want to Be a Wizard: The First Book in the Young Wizards Series, The Grey King (The Dark is Rising Sequence) | 1 |
| book::316::0 | What books have you recommended to me before? | raw | false | Die HÃ?Â¤upter meiner Lieben. | 1 |
| book::317::0 | What books have you recommended to me before? | raw | false | Your Pregnancy: Week by Week (Your Pregnancy Series), Fat Land: How Americans Became the Fattest People in the World | 1 |
| book::318::0 | What books have you recommended to me before? | raw | true | Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It, A Mind of Its Own: A Cultural History of the Penis, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| book::319::0 | What books have you recommended to me before? | raw | false | Cinematherapy : The Girl's Guide to Movies for Every Mood, Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks, The Watcher's Guide 2 (Buffy the Vampire Slayer) | 1 |
| book::320::0 | What books have you recommended to me before? | raw | false | Parliament of Whores: A Lone Humorist Attempts to Explain the Entire U.S. Government | 1 |
| book::321::0 | What books have you recommended to me before? | raw | false | Bitch: In Praise of Difficult Women, Brothel: Mustang Ranch and Its Women, The Woman Warrior : Memoirs of a Girlhood Among Ghosts, Lakota Woman | 1 |
| book::322::0 | What books have you recommended to me before? | raw | false | The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss, Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements | 1 |
| book::323::0 | What books have you recommended to me before? | raw | true | If the Buddha Dated: A Handbook for Finding Love on a Spiritual Path, The Kiss | 1 |
| book::324::0 | What books have you recommended to me before? | raw | false | Mansfield Park (Penguin Classics) | 1 |
| book::325::0 | What books have you recommended to me before? | raw | false | American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library), Diet for a New America, The Woman Warrior : Memoirs of a Girlhood Among Ghosts, The Sorcerer's Companion: A Guide to the Magical World of Harry Potter | 1 |
| book::326::0 | What books have you recommended to me before? | raw | true | The Universe in a Nutshell, A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition, Cosmos | 1 |
| book::327::0 | What books have you recommended to me before? | raw | false | Enigma., GefÃ?Â¤hrliche Geliebte. | 1 |
| book::328::0 | What books have you recommended to me before? | raw | false | The Curious Sofa: A Pornographic Work by Ogdred Weary, The Iron Tonic: Or, A Winter Afternoon in Lonely Valley | 1 |
| book::329::0 | What books have you recommended to me before? | raw | true | Tuesdays with Morrie: An Old Man, a Young Man, and Life's Greatest Lesson, The Color of Water: A Black Man's Tribute to His White Mother | 1 |
| book::330::0 | What books have you recommended to me before? | raw | false | Romeo and Juliet (Bantam Classic), A Midsummer Nights Dream (Bantam Classic) | 1 |
| book::331::0 | What books have you recommended to me before? | raw | false | Brothel: Mustang Ranch and Its Women | 1 |
| book::332::0 | What books have you recommended to me before? | raw | false | Angels and Demons | 1 |
| book::333::0 | What books have you recommended to me before? | raw | true | In the Name of Love : Ann Rule's Crime Files Volume 4 (Ann Rule's Crime Files), The Jane Austen Book Club | 1 |
| book::334::0 | What books have you recommended to me before? | raw | true | Gianna: Aborted... and Lived to Tell About It (Living Books), The Kiss | 1 |
| book::335::0 | What books have you recommended to me before? | raw | false | We're Right, They're Wrong: A Handbook for Spirited Progressives, The Prince, Bush at War | 1 |
| book::336::0 | What books have you recommended to me before? | raw | true | A Streetcar Named Desire, Rosencrantz & Guildenstern Are Dead | 1 |
| book::337::0 | What books have you recommended to me before? | raw | true | Love in the Time of Cholera (Penguin Great Books of the 20th Century), Madame Bovary: Provincial Lives (Penguin Classics) | 1 |
| book::338::0 | What books have you recommended to me before? | raw | false | Angela's Ashes: A Memoir | 1 |
| book::339::0 | What books have you recommended to me before? | raw | false | High Tide in Tucson : Essays from Now or Never | 1 |
| book::340::0 | What books have you recommended to me before? | raw | false | Cats and Their Women | 1 |
| book::341::0 | What books have you recommended to me before? | raw | false | The Perfect Storm : A True Story of Men Against the Sea, The Man Who Listens to Horses | 1 |
| book::342::0 | What books have you recommended to me before? | raw | false | Bitch: In Praise of Difficult Women, The Sorcerer's Companion: A Guide to the Magical World of Harry Potter | 1 |
| book::343::0 | What books have you recommended to me before? | raw | true | The Prince, We're Right, They're Wrong: A Handbook for Spirited Progressives | 1 |
| book::344::0 | What books have you recommended to me before? | raw | false | Book of Tea, The Te of Piglet, What Should I Do with My Life? | 1 |
| book::345::0 | What books have you recommended to me before? | raw | false | Diet for a Small Planet (20th Anniversary Edition), New Vegetarian: Bold and Beautiful Recipes for Every Occasion, Fix-It and Forget-It Cookbook: Feasting with Your Slow Cooker | 1 |
| book::346::0 | What books have you recommended to me before? | raw | false | The Vagina Monologues: The V-Day Edition, Mike Nelson's Movie Megacheese | 1 |
| book::347::0 | What books have you recommended to me before? | raw | false | Death: The High Cost of Living, Wildlife Preserves, Scientific Progress Goes 'Boink':  A Calvin and Hobbes Collection | 1 |
| book::348::0 | What books have you recommended to me before? | raw | true | So You Want to Be a Wizard: The First Book in the Young Wizards Series, Prince Caspian | 1 |
| book::349::0 | What books have you recommended to me before? | raw | false | Book of Virtues | 1 |
| book::350::0 | What books have you recommended to me before? | raw | false | Uncle Shelby's ABZ Book: A Primer for Adults Only | 1 |
| book::351::0 | What books have you recommended to me before? | raw | false | Wicca: A Guide for the Solitary Practitioner, Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| book::352::0 | What books have you recommended to me before? | raw | false | Ghost World | 1 |
| book::353::0 | What books have you recommended to me before? | raw | false | Talking to Heaven: A Medium's Message of Life After Death, Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| book::354::0 | What books have you recommended to me before? | raw | false | Different Seasons, Clan of the Cave Bear | 1 |
| book::355::0 | What books have you recommended to me before? | raw | false | To Ride a Silver Broomstick: New Generation Witchcraft, The Mothman Prophecies, SEAT OF THE SOUL | 1 |
| book::356::0 | What books have you recommended to me before? | raw | false | The Te of Piglet | 1 |
| book::357::0 | What books have you recommended to me before? | raw | false | Fraud: Essays | 1 |
| book::358::0 | What books have you recommended to me before? | raw | false | The Law, The Cases That Haunt Us | 1 |
| book::359::0 | What books have you recommended to me before? | raw | false | A 5th Portion of Chicken Soup for the Soul : 101 Stories to Open the Heart and Rekindle the Spirit, The Four Agreements: A Practical Guide to Personal Freedom | 1 |
| book::360::0 | What books have you recommended to me before? | raw | false | A Night Without Armor : Poems, Songs of Innocence and Songs of Experience (Dover Thrift Editions), Ain't I A Woman!: A Book of Women's Poetry from Around the World | 1 |
| book::361::0 | What books have you recommended to me before? | raw | true | What to Expect When You're Expecting (Revised Edition), Your Pregnancy: Week by Week (Your Pregnancy Series) | 1 |
| book::362::0 | What books have you recommended to me before? | raw | false | Under the Tuscan Sun | 1 |
| book::363::0 | What books have you recommended to me before? | raw | false | Divorce Your Car! : Ending the Love Affair with the Automobile | 1 |
| book::364::0 | What books have you recommended to me before? | raw | false | The Prayer of Jabez: Breaking Through to the Blessed Life, The Screwtape Letters | 1 |
| book::365::0 | What books have you recommended to me before? | raw | false | Ciao, America: An Italian Discovers the U.S | 1 |
| book::366::0 | What books have you recommended to me before? | raw | false | The Prince, Bush at War, Stupid White Men ...and Other Sorry Excuses for the State of the Nation!, The O'Reilly Factor: The Good, the Bad, and the Completely Ridiculous in American Life | 1 |
| book::367::0 | What books have you recommended to me before? | raw | false | Good in Bed, A Painted House | 1 |
| book::368::0 | What books have you recommended to me before? | raw | false | Team Rodent : How Disney Devours the World | 1 |
| book::369::0 | What books have you recommended to me before? | raw | true | A Rage To Kill and Other True Cases : Anne Rule's Crime Files, Vol. 6 (Ann Rule's Crime Files), DEAD BY SUNSET : DEAD BY SUNSET | 1 |
| book::370::0 | What books have you recommended to me before? | raw | false | Self Matters : Creating Your Life from the Inside Out, Nickel and Dimed: On (Not) Getting By in America | 1 |
| book::371::0 | What books have you recommended to me before? | raw | false | Farewell to Manzanar: A True Story of Japanese American Experience During and  After the World War II Internment | 1 |
| book::372::0 | What books have you recommended to me before? | raw | false | Wizard of Oz Postcards in Full Color (Card Books) | 1 |
| book::373::0 | What books have you recommended to me before? | raw | false | Body for Life: 12 Weeks to Mental and Physical Strength, Make the Connection: Ten Steps to a Better Body and a Better Life | 1 |
| book::374::0 | What books have you recommended to me before? | raw | false | Savage Inequalities: Children in America's Schools, The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child, Book of Virtues | 1 |
| book::375::0 | What books have you recommended to me before? | raw | true | The Street Lawyer, Illusions | 1 |
| book::376::0 | What books have you recommended to me before? | raw | false | Their eyes were watching God: A novel, The Prayer of Jabez: Breaking Through to the Blessed Life | 1 |
| book::377::0 | What books have you recommended to me before? | raw | false | The Snow Leopard (Penguin Nature Classics) | 1 |
| book::378::0 | What books have you recommended to me before? | raw | false | Take Care of Yourself: The Complete Illustrated Guide to Medical Self-Care, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| book::379::0 | What books have you recommended to me before? | raw | true | Girl with a Pearl Earring, The Secret Life of Bees | 1 |
| book::380::0 | What books have you recommended to me before? | raw | false | Anger: Wisdom for Cooling the Flames | 1 |
| book::381::0 | What books have you recommended to me before? | raw | true | The Law, A Civil Action, Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States | 1 |
| book::382::0 | What books have you recommended to me before? | raw | false | Peace Is Every Step: The Path of Mindfulness in Everyday Life, Many Lives, Many Masters | 1 |
| book::383::0 | What books have you recommended to me before? | raw | false | The Greatest Show Off Earth, Strata | 1 |
| book::384::0 | What books have you recommended to me before? | raw | false | HITCHHIK GD GALAXY (Hitchhiker's Trilogy (Paperback)), So Long and Thanks for all the Fish | 1 |
| book::385::0 | What books have you recommended to me before? | raw | false | Small Wonder : Essays | 1 |
| book::386::0 | What books have you recommended to me before? | raw | true | All Through The Night : A Suspense Story, The Mists of Avalon | 1 |
| book::387::0 | What books have you recommended to me before? | raw | true | SHIPPING NEWS | 1 |
| book::388::0 | What books have you recommended to me before? | raw | false | The Hot Zone, The Coming Plague: Newly Emerging Diseases in a World Out of Balance, An Anthropologist on Mars: Seven Paradoxical Tales | 1 |
| book::389::0 | What books have you recommended to me before? | raw | true | Anne Frank: The Diary of a Young Girl | 1 |
| book::390::0 | What books have you recommended to me before? | raw | false | Selected Poems (Dover Thrift Edition), Selected Poems (Dover Thrift Editions) | 1 |
| book::391::0 | What books have you recommended to me before? | raw | true | Harry Potter and the Sorcerer's Stone (Harry Potter (Paperback)), Harry Potter and the Goblet of Fire (Book 4) | 1 |
| book::392::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Oprah's Book Club), Lady Chatterley's Lover | 1 |
| book::393::0 | What books have you recommended to me before? | raw | false | The Girlfriends' Guide to Pregnancy, Ophelia Speaks : Adolescent Girls Write About Their Search for Self | 1 |
| book::394::0 | What books have you recommended to me before? | raw | false | Uncle Shelby's ABZ Book: A Primer for Adults Only, Book of Virtues | 1 |
| book::395::0 | What books have you recommended to me before? | raw | false | Parliament of Whores: A Lone Humorist Attempts to Explain the Entire U.S. Government | 1 |
| book::396::0 | What books have you recommended to me before? | raw | false | Divorce Your Car! : Ending the Love Affair with the Automobile | 1 |
| book::397::0 | What books have you recommended to me before? | raw | false | Small Wonder : Essays, Small Wonder: Essays | 1 |
| book::398::0 | What books have you recommended to me before? | raw | false | We're Right, They're Wrong: A Handbook for Spirited Progressives, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| book::399::0 | What books have you recommended to me before? | raw | false | Good in Bed, Snow Falling on Cedars | 1 |
| book::400::0 | What books have you recommended to me before? | raw | false | Death of A Salesman | 1 |
| book::401::0 | What books have you recommended to me before? | raw | false | The Street Lawyer, The Gunslinger (The Dark Tower, Book 1) | 1 |
| book::402::0 | What books have you recommended to me before? | raw | false | Chobits (Chobits) | 1 |
| book::403::0 | What books have you recommended to me before? | raw | true | All Through The Night : A Suspense Story, Another Roadside Attraction | 1 |
| book::404::0 | What books have you recommended to me before? | raw | true | Harry Potter and the Prisoner of Azkaban (Book 3), Harry Potter and the Chamber of Secrets (Book 2) | 1 |
| book::405::0 | What books have you recommended to me before? | raw | false | The Mothman Prophecies, Many Lives, Many Masters, Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| book::406::0 | What books have you recommended to me before? | raw | false | The Demon-Haunted World: Science As a Candle in the Dark, A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition | 1 |
| book::407::0 | What books have you recommended to me before? | raw | false | More Than Complete Hitchhiker's Guide, The original Hitchhiker radio scripts | 1 |
| book::408::0 | What books have you recommended to me before? | raw | true | The Teenage Liberation Handbook: How to Quit School and Get a Real Life and Education, Chinese Cinderella: The True Story of an Unwanted Daughter (Laurel-Leaf Books), Go Ask Alice (Avon/Flare Book) | 1 |
| book::409::0 | What books have you recommended to me before? | raw | false | The Doubtful Guest, The Philosophy of Andy Warhol | 1 |
| book::410::0 | What books have you recommended to me before? | raw | false | A Dangerous Fortune | 1 |
| book::411::0 | What books have you recommended to me before? | raw | true | Romeo and Juliet (Dover Thrift Editions), Four Major Plays: A Doll House, the Wild Duck, Hedda Gabler, the Master Builder (Signet Classics (Paperback)), A Streetcar Named Desire | 1 |
| book::412::0 | What books have you recommended to me before? | raw | false | The Golden Compass (His Dark Materials, Book 1), Harry Potter and the Order of the Phoenix (Book 5) | 1 |
| book::413::0 | What books have you recommended to me before? | raw | true | So You Want to Be a Wizard: The First Book in the Young Wizards Series, The Magician's Nephew | 1 |
| book::414::0 | What books have you recommended to me before? | raw | false | The Girlfriends' Guide to Pregnancy | 1 |
| book::415::0 | What books have you recommended to me before? | raw | false | In the Heart of the Sea: The Tragedy of the Whaleship Essex, Hiroshima | 1 |
| book::416::0 | What books have you recommended to me before? | raw | false | The Law, One L : The Turbulent True Story of a First Year at Harvard Law School | 1 |
| book::417::0 | What books have you recommended to me before? | raw | true | The Kiss, A Natural History of the Senses, What to Expect the First Year | 1 |
| book::418::0 | What books have you recommended to me before? | raw | false | 9-11 | 1 |
| book::419::0 | What books have you recommended to me before? | raw | false | The Moonstone (Penguin Classics) | 1 |
| book::420::0 | What books have you recommended to me before? | raw | true | A Civil Action, The Cases That Haunt Us | 1 |
| book::421::0 | What books have you recommended to me before? | raw | false | Nine Parts of Desire: The Hidden World of Islamic Women | 1 |
| book::422::0 | What books have you recommended to me before? | raw | true | The Sweet Potato Queens' Book of Love | 1 |
| book::423::0 | What books have you recommended to me before? | raw | false | Sonnets from the Portuguese and Other Poems (Dover Thrift Editions), 100 Selected Poems by E. E. Cummings | 1 |
| book::424::0 | What books have you recommended to me before? | raw | false | The Prayer of Jabez: Breaking Through to the Blessed Life, The Case for Christ:  A Journalist's Personal Investigation of the Evidence for Jesus | 1 |
| book::425::0 | What books have you recommended to me before? | raw | false | To Kill a Mockingbird | 1 |
| book::426::0 | What books have you recommended to me before? | raw | false | Different Seasons | 1 |
| book::427::0 | What books have you recommended to me before? | raw | false | The Elements of Style, Fourth Edition | 1 |
| book::428::0 | What books have you recommended to me before? | raw | false | The Grey King (The Dark is Rising Sequence), The Magician's Nephew | 1 |
| book::429::0 | What books have you recommended to me before? | raw | false | Naked, Lies and the Lying Liars Who Tell Them: A Fair and Balanced Look at the Right | 1 |
| book::430::0 | What books have you recommended to me before? | raw | false | HEARTBURN, Behind the Scenes at the Museum | 1 |
| book::431::0 | What books have you recommended to me before? | raw | false | Diet for a New America, Brothel: Mustang Ranch and Its Women | 1 |
| book::432::0 | What books have you recommended to me before? | raw | false | Das Hotel New Hampshire | 1 |
| book::433::0 | What books have you recommended to me before? | raw | false | The Blue Day Book | 1 |
| book::434::0 | What books have you recommended to me before? | raw | false | In the Heart of the Sea: The Tragedy of the Whaleship Essex, Hiroshima, Guns, Germs, and Steel: The Fates of Human Societies | 1 |
| book::435::0 | What books have you recommended to me before? | raw | false | The Fellowship of the Ring (The Lord of the Rings, Part 1) | 1 |
| book::436::0 | What books have you recommended to me before? | raw | true | Nobilta. Commissario Brunettis siebter Fall., WLD ACCORDNG GARP | 1 |
| book::437::0 | What books have you recommended to me before? | raw | false | Red Dwarf, The Greatest Show Off Earth | 1 |
| book::438::0 | What books have you recommended to me before? | raw | false | The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child | 1 |
| book::439::0 | What books have you recommended to me before? | raw | false | Blind Faith, DEAD BY SUNSET : DEAD BY SUNSET | 1 |
| book::440::0 | What books have you recommended to me before? | raw | false | Uncle Shelby's ABZ Book: A Primer for Adults Only, Book of Virtues | 1 |
| book::441::0 | What books have you recommended to me before? | raw | true | My Family and Other Animals. | 1 |
| book::442::0 | What books have you recommended to me before? | raw | false | Dude, Where's My Country?, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| book::443::0 | What books have you recommended to me before? | raw | true | What Should I Do with My Life?, Lila: An Inquiry Into Morals | 1 |
| book::444::0 | What books have you recommended to me before? | raw | false | El Senor De Los Anillos: LA Comunidad Del Anillo (Lord of the Rings (Spanish)), Die Gefahrten I | 1 |
| book::445::0 | What books have you recommended to me before? | raw | true | Even Cowgirls Get the Blues, The Mists of Avalon, All Through The Night : A Suspense Story | 1 |
| book::446::0 | What books have you recommended to me before? | raw | false | Who Moved My Cheese? An Amazing Way to Deal with Change in Your Work and in Your Life, Self Matters : Creating Your Life from the Inside Out | 1 |
| book::447::0 | What books have you recommended to me before? | raw | false | Fat Land: How Americans Became the Fattest People in the World, Body for Life: 12 Weeks to Mental and Physical Strength, Dr. Atkins' New Diet Revolution | 1 |
| book::448::0 | What books have you recommended to me before? | raw | false | Enigma., Mansfield Park (Penguin Classics) | 1 |
| book::449::0 | What books have you recommended to me before? | raw | true | Harry Potter and the Sorcerer's Stone (Harry Potter (Paperback)), The Golden Compass (His Dark Materials, Book 1) | 1 |
| book::450::0 | What books have you recommended to me before? | raw | true | Chinese Cinderella: The True Story of an Unwanted Daughter (Laurel-Leaf Books), Farewell to Manzanar: A True Story of Japanese American Experience During and  After the World War II Internment, The Teenage Liberation Handbook: How to Quit School and Get a Real Life and Education | 1 |
| book::451::0 | What books have you recommended to me before? | raw | false | An Anthropologist on Mars: Seven Paradoxical Tales | 1 |
| book::452::0 | What books have you recommended to me before? | raw | false | Lies and the Lying Liars Who Tell Them: A Fair and Balanced Look at the Right, Seinlanguage, Naked | 1 |
| book::453::0 | What books have you recommended to me before? | raw | false | Who Moved My Cheese? An Amazing Way to Deal with Change in Your Work and in Your Life, Fish! A Remarkable Way to Boost Morale and Improve Results | 1 |
| book::454::0 | What books have you recommended to me before? | raw | false | The Case for Christ:  A Journalist's Personal Investigation of the Evidence for Jesus, Plain and Simple : A Journey to the Amish (Ohio) | 1 |
| book::455::0 | What books have you recommended to me before? | raw | false | One Hundred Ways for a Cat to Train Its Human, ALL MY PATIENTS ARE UNDER THE BED | 1 |
| book::456::0 | What books have you recommended to me before? | raw | true | Good Faeries Bad Faeries, Lust for Life, Why Cats Paint: A Theory of Feline Aesthetics | 1 |
| book::457::0 | What books have you recommended to me before? | raw | false | There Are No Children Here: The Story of Two Boys Growing Up in the Other America, Diet for a New America, American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library) | 1 |
| book::458::0 | What books have you recommended to me before? | raw | false | Who Moved My Cheese? An Amazing Way to Deal with Change in Your Work and in Your Life, Nickel and Dimed: On (Not) Getting By in America, Nickel and Dimed: On (Not) Getting By in America, Fish! A Remarkable Way to Boost Morale and Improve Results | 1 |
| book::459::0 | What books have you recommended to me before? | raw | false | Acqua Alta | 1 |
| book::460::0 | What books have you recommended to me before? | raw | true | Book of Virtues, Savage Inequalities: Children in America's Schools, The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child | 1 |
| book::461::0 | What books have you recommended to me before? | raw | true | The Dark Side of the Light Chasers: Reclaiming Your Power, Creativity, Brilliance, and Dreams, Man's Search for Meaning: An Introduction to Logotherapy | 1 |
| book::462::0 | What books have you recommended to me before? | raw | false | The Universe in a Nutshell, A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition, An Anthropologist on Mars: Seven Paradoxical Tales, My Family and Other Animals. | 1 |
| book::463::0 | What books have you recommended to me before? | raw | false | Take Care of Yourself: The Complete Illustrated Guide to Medical Self-Care, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| book::464::0 | What books have you recommended to me before? | raw | false | Bitter Harvest, EVERYTHING SHE EVER WANTED | 1 |
| book::465::0 | What books have you recommended to me before? | raw | false | SHIPPING NEWS, Behind the Scenes at the Museum, MY SWEET AUDRINA, Das Hotel New Hampshire | 1 |
| book::466::0 | What books have you recommended to me before? | raw | false | El Senor De Los Anillos: El Retorno Del Rey (Tolkien, J. R. R. Lord of the Rings. 3.), The Fellowship of the Ring | 1 |
| book::467::0 | What books have you recommended to me before? | raw | false | Snow Falling on Cedars | 1 |
| book::468::0 | What books have you recommended to me before? | raw | true | A Fever in the Heart : Ann Rule's Crime Files, Volume III, Book Lust: Recommended Reading for Every Mood, Moment, and Reason | 1 |
| book::469::0 | What books have you recommended to me before? | raw | true | The Perfect Storm : A True Story of Men Against the Sea, Divorce Your Car! : Ending the Love Affair with the Automobile | 1 |
| book::470::0 | What books have you recommended to me before? | raw | false | Flow: The Psychology of Optimal Experience | 1 |
| book::471::0 | What books have you recommended to me before? | raw | false | Creative Companion: How to Free Your Creative Spirit, The Tipping Point: How Little Things Can Make a Big Difference | 1 |
| book::472::0 | What books have you recommended to me before? | raw | false | Selected Poems (Dover Thrift Edition), Beowulf: A New Verse Translation, Ain't I A Woman!: A Book of Women's Poetry from Around the World | 1 |
| book::473::0 | What books have you recommended to me before? | raw | false | A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition, The Demon-Haunted World: Science As a Candle in the Dark, Cosmos | 1 |
| book::474::0 | What books have you recommended to me before? | raw | false | Ciao, America: An Italian Discovers the U.S | 1 |
| book::475::0 | What books have you recommended to me before? | raw | true | Lies and the Lying Liars Who Tell Them: A Fair and Balanced Look at the Right, Mama Makes Up Her Mind: And Other Dangers of Southern Living | 1 |
| book::476::0 | What books have you recommended to me before? | raw | false | Diet for a Small Planet (20th Anniversary Edition), Chocolate: The Consuming Passion | 1 |
| book::477::0 | What books have you recommended to me before? | raw | false | Anna Karenina (Penguin Classics) | 1 |
| book::478::0 | What books have you recommended to me before? | raw | true | The Girlfriends' Guide to Pregnancy | 1 |
| book::479::0 | What books have you recommended to me before? | raw | true | A Civil Action | 1 |
| book::480::0 | What books have you recommended to me before? | raw | false | The Meaning Of Life, Wicca: A Guide for the Solitary Practitioner, Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| book::481::0 | What books have you recommended to me before? | raw | false | The Curious Sofa: A Pornographic Work by Ogdred Weary | 1 |
| book::482::0 | What books have you recommended to me before? | raw | false | The Fellowship of the Ring (The Lord of the Rings, Part 1), Die Gefahrten I | 1 |
| book::483::0 | What books have you recommended to me before? | raw | false | Another Roadside Attraction, Left Behind: A Novel of the Earth's Last Days (Left Behind #1), Dark Water (Mira Romantic Suspense) | 1 |
| book::484::0 | What books have you recommended to me before? | raw | false | Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It, An Anthropologist on Mars: Seven Paradoxical Tales | 1 |
| book::485::0 | What books have you recommended to me before? | raw | true | Angels and Demons, Nobilta. Commissario Brunettis siebter Fall., Different Seasons | 1 |
| book::486::0 | What books have you recommended to me before? | raw | false | A Night to Remember | 1 |
| book::487::0 | What books have you recommended to me before? | raw | false | The Devil in the White City : Murder, Magic, and Madness at the Fair That Changed America (Illinois), Hiroshima, Seabiscuit: An American Legend | 1 |
| book::488::0 | What books have you recommended to me before? | raw | false | The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them, Savage Inequalities: Children in America's Schools | 1 |
| book::489::0 | What books have you recommended to me before? | raw | true | Chobits Vol.1, Ghost World | 1 |
| book::490::0 | What books have you recommended to me before? | raw | true | Romeo and Juliet (Bantam Classic), The Importance of Being Earnest (Dover Thrift Editions) | 1 |
| book::491::0 | What books have you recommended to me before? | raw | false | Creative Companion: How to Free Your Creative Spirit, The Anatomy of Motive : The FBI's Legendary Mindhunter Explores the Key to Understanding and Catching Violent Criminals, The Psychologist's Book of Self-Tests: 25 Love, Sex, Intelligence, Career, and Personality Tests Developed by Professionals to Reveal the Real You | 1 |
| book::492::0 | What books have you recommended to me before? | raw | true | What to Expect When You're Expecting (Revised Edition) | 1 |
| book::493::0 | What books have you recommended to me before? | raw | true | Angels and Demons, The Clan of the Cave Bear : a novel | 1 |
| book::494::0 | What books have you recommended to me before? | raw | false | The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child | 1 |
| book::495::0 | What books have you recommended to me before? | raw | true | What to Expect the First Year, The Girlfriends' Guide to Pregnancy, The Kiss | 1 |
| book::496::0 | What books have you recommended to me before? | raw | false | The Philosophy of Andy Warhol, Wizard of Oz Postcards in Full Color (Card Books) | 1 |
| book::497::0 | What books have you recommended to me before? | raw | false | American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library), Bitch: In Praise of Difficult Women | 1 |
| book::498::0 | What books have you recommended to me before? | raw | false | A Rage To Kill and Other True Cases : Anne Rule's Crime Files, Vol. 6 (Ann Rule's Crime Files) | 1 |
| book::499::0 | What books have you recommended to me before? | raw | false | 8 Weeks to Optimum Health, Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements | 1 |
| events::0::0 | What seven-day event perfectly aligns with its location? | raw | false | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::1::0 | What time is the event that expects two hundred people? | raw | true | nextnext week Wednesday 9:00 AM | 2 |
| events::2::0 | What is the timing for the event that lasts seven days? | raw | true | 2024-10-18 Friday 09:00 | 2 |
| events::3::0 | What is the schedule for the event that lasts six days? | raw | true | nextnext week Tuesday 7:00 PM | 2 |
| events::4::0 | What time is the event taking place at that location in Washington, DC? | raw | true | next week Sunday 7:00 PM | 3 |
| events::5::0 | Which venue would be suitable for an event that accommodates nine hundred people? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::6::0 | Which event corresponds to the location described for the activity planned for the week after next Thursday at 9:00 AM? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::7::0 | What six-day activity corresponds to its location description? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::8::0 | What seven-week activity fits the description of its location? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::9::0 | Which location description matches the event planned for the week after next Sunday at 2:00 PM? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::10::0 | What is the event location description for the activity scheduled on October 12, 2024, at 19:00? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::11::0 | What activity lasts for eight days and corresponds with its location description? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::12::0 | What time is the event that is expected to have eight hundred people? | raw | true | nextnext week Tuesday 9:00 AM | 2 |
| events::13::0 | What time is the event at that location in Las Vegas, NV? | raw | true | 2024-10-11 Friday 14:00 | 2 |
| events::14::0 | What event location description corresponds to the activity scheduled for October 17, 2024, at 9:00? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::15::0 | Which event that hosts four hundred people fits the description of its location? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::16::0 | Which event location corresponds to the activity taking place next week on Sunday at 9:00 AM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::17::0 | What is the scheduled time for the event that accommodates four hundred people? | raw | false | 2024-10-11 Friday 14:00 | 2 |
| events::18::0 | Which venue corresponds to the event for nine hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::19::0 | What three-week activity matches the description of its location? | raw | false | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::20::0 | What time is the event taking place at the location in Austin, TX? | raw | true | nextnext week Tuesday 2:00 PM | 2 |
| events::21::0 | Which location is set to host an event on October 16, 2024, at 14:00? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::22::0 | What venue would be suitable for an event accommodating six hundred people? | raw | false | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::23::0 | What time is the event that will have around seven hundred people? | raw | true | next week Friday 9:00 AM | 2 |
| events::24::0 | What time will the event take place in Atlanta, GA? | raw | false | next week Saturday 9:00 AM | 2 |
| events::25::0 | Which event location description corresponds to the activity planned for October 17, 2024, at 19:00? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::26::0 | What is the timing for the activity that involves three hundred people? | raw | true | nextnext week Thursday 2:00 PM | 2 |
| events::27::0 | What time is the event that has a scale of eight hundred people? | raw | true | 2024-10-15 Tuesday 14:00 | 2 |
| events::28::0 | Which location is hosting the event planned for next week on Thursday at 2:00 PM? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::29::0 | What time is the event that has an expected attendance of eight hundred people? | raw | true | 2024-10-08 Tuesday 09:00 | 2 |
| events::30::0 | What time is the event for the nine hundred people? | raw | true | 2024-10-09 Wednesday 09:00 | 2 |
| events::31::0 | What time is the event happening in Boston, MA? | raw | false | 2024-10-17 Thursday 19:00 | 2 |
| events::32::0 | What is the event location description for the activity scheduled on October 13, 2024, at 7:00 PM? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::33::0 | Which venue would be suitable for an event with around three hundred attendees? | raw | false | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::34::0 | What time is the activity scheduled for that lasts five days? | raw | true | nextnext week Monday 7:00 PM | 2 |
| events::35::0 | Which venue description fits an activity scale of two hundred people? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::36::0 | Which venue description would be suitable for an event that accommodates three hundred people? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::37::0 | Which venue would be suitable for an event that accommodates five hundred people? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::38::0 | Which event venue is suitable for an activity that accommodates seven hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::39::0 | What time is the event taking place at that location in Los Angeles, CA? | raw | true | next week Sunday 7:00 PM | 2 |
| events::40::0 | What time is the event happening in San Francisco, CA? | raw | true | 2024-10-14 Monday 09:00 | 2 |
| events::41::0 | What event coincides with the location description for next week Sunday at 2:00 PM? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::42::0 | Which event location description corresponds to the activity planned for the week after next Wednesday at 2:00 PM? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::43::0 | What one-day activity aligns perfectly with its location description? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::44::0 | Which location is designated for the event happening at 2:00 PM the week after next Sunday? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::45::0 | What is the schedule for the event that lasts four days? | raw | true | next week Sunday 9:00 AM | 2 |
| events::46::0 | What time is the event scheduled for with nine hundred attendees? | raw | true | 2024-10-14 Monday 19:00 | 2 |
| events::47::0 | What time is the event scheduled for at that location in Las Vegas, NV? | raw | false | next week Saturday 7:00 PM | 2 |
| events::48::0 | What time is the event that's expected to have around two hundred people? | raw | false | 2024-10-12 Saturday 14:00 | 2 |
| events::49::0 | When does the activity that lasts nine weeks take place? | raw | true | next week Saturday 7:00 PM | 2 |
| events::50::0 | Which event description corresponds to the venue that can accommodate eight hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::51::0 | What is the timeframe for the activity that lasts two weeks? | raw | true | next week Saturday 7:00 PM | 2 |
| events::52::0 | What activity that lasts five weeks fits the description of its location? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::53::0 | What event location would be suitable for an activity involving eight hundred people? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::54::0 | What one-day activity corresponds with the description of its location? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::55::0 | What time is the event expected to start for a gathering of two hundred people? | raw | true | 2024-10-11 Friday 09:00 | 2 |
| events::56::0 | What kind of event location would be suitable for an activity with around six hundred people? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::57::0 | What event is happening at the location described for the one scheduled for the week after next Saturday at 7:00 PM? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::58::0 | What time does the event start in Austin, TX? | raw | false | nextnext week Tuesday 2:00 PM | 2 |
| events::59::0 | What is the timeframe for the activity that lasts a week? | raw | true | 2024-10-10 Thursday 19:00 | 2 |
| events::60::0 | What activity lasts nine weeks and corresponds with the description of its location? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::61::0 | What time does the event take place in Chicago, IL? | raw | true | 2024-10-18 Friday 09:00 | 2 |
| events::62::0 | What time does the event that lasts three days start? | raw | false | nextnext week Thursday 7:00 PM | 2 |
| events::63::0 | What activity that lasts nine weeks matches its location description? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::64::0 | What activity lasts for nine weeks and has a location that fits its description? | raw | true | Known for its historical significance and the Liberty Bell. | 2 |
| events::65::0 | What time is the event happening in San Francisco, CA? | raw | false | 2024-10-13 Sunday 19:00 | 2 |
| events::66::0 | What is the timeframe for the activity that lasts seven weeks? | raw | true | 2024-10-12 Saturday 09:00 | 2 |
| events::67::0 | What time is the event taking place in Seattle, WA? | raw | true | nextnext week Monday 2:00 PM | 2 |
| events::68::0 | What time is the event for nine hundred people? | raw | false | nextnext week Thursday 2:00 PM | 2 |
| events::69::0 | What is the timing for the event with two hundred attendees? | raw | false | next week Sunday 2:00 PM | 2 |
| events::70::0 | What is the event location description for the activity set for October 17, 2024, at 9:00? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::71::0 | What activity matches the description of the location for that two-week event? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::72::0 | What is the event location for the activity planned on October 15, 2024, at 14:00? | raw | false | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::73::0 | Which venue would be suitable for hosting an event with around seven hundred attendees? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::74::0 | Which venue hosts events for nine hundred people? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::75::0 | What time is the event happening in Austin, TX? | raw | true | nextnext week Tuesday 7:00 PM | 2 |
| events::76::0 | Which venue fits the description for an event with a capacity of seven hundred people? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::77::0 | What time is the event that will have six hundred people attending? | raw | true | 2024-10-08 Tuesday 14:00 | 2 |
| events::78::0 | What time is the event that will have three hundred people attending? | raw | true | 2024-10-09 Wednesday 14:00 | 2 |
| events::79::0 | What is the time for the event that will accommodate five hundred people? | raw | true | nextnext week Wednesday 2:00 PM | 2 |
| events::80::0 | Which event location description corresponds to the activity set for October 12, 2024, at 2:00 PM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::81::0 | Which venue would be suitable for an event accommodating around two hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::82::0 | Which event corresponds to the location description for the activity planned for the week after next Friday at 7:00 PM? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::83::0 | Which activity lasts for eight days and fits the description of its location? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::84::0 | Is there an activity that lasts six weeks and has a location that matches its description? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::85::0 | Which venue would be suitable for an event that accommodates around seven hundred people? | raw | false | Known for its theme parks, including Walt Disney World. | 2 |
| events::86::0 | What time is the event expected to take place that will have around seven hundred people attending? | raw | false | 2024-10-18 Friday 19:00 | 2 |
| events::87::0 | What four-day event fits the description of its location? | raw | false | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::88::0 | Which venue is suitable for an activity involving around one hundred people? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| events::89::0 | Which nine-day activity aligns with the description of its location? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 3 |
| events::90::0 | What venue would be suitable for an event with a scale of four hundred people? | raw | false | Famous for its entertainment, casinos, and vibrant nightlife. | 3 |
| events::91::0 | What time is the event scheduled to start at that location in Los Angeles, CA? | raw | true | next week Sunday 7:00 PM | 2 |
| events::92::0 | How long is the activity that lasts for nine weeks? | raw | true | 2024-10-11 Friday 14:00 | 2 |
| events::93::0 | What time does the event in San Francisco, CA start? | raw | true | nextnext week Wednesday 2:00 PM | 3 |
| events::94::0 | Which event corresponds to the location description for the activity planned for next week on Thursday at 9:00 AM? | raw | true | Known for its theme parks, including Walt Disney World. | 3 |
| events::95::0 | What time does the event take place at the location in San Francisco, CA? | raw | false | next week Monday 2:00 PM | 2 |
| events::96::0 | What event corresponds to the location description for the activity set to take place the week after next Saturday at 9:00 AM? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 4 |
| events::97::0 | Which five-day event corresponds to the description of its location? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| events::98::0 | What time is the event that will have around six hundred people? | raw | true | 2024-10-11 Friday 09:00 | 2 |
| events::99::0 | Which event location would be suitable for an activity that accommodates around five thousand people? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::100::0 | Which location aligns with an event that accommodates seven hundred people? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::101::0 | When does an event that lasts for nine days take place? | raw | true | next week Friday 2:00 PM | 2 |
| events::102::0 | How long does the activity that lasts for eight weeks take? | raw | true | nextnext week Tuesday 2:00 PM | 2 |
| events::103::0 | What time does the event in Boston, MA start? | raw | true | next week Monday 2:00 PM | 2 |
| events::104::0 | Which venue description is suitable for an event with three hundred attendees? | raw | false | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::105::0 | Which location is designated for the event taking place at 9:00 AM the week after next Saturday? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::106::0 | What time does the event take place in Orlando, FL? | raw | true |  week Sunday 2:00 PM | 2 |
| events::107::0 | What time is the event taking place at that location in Portland, OR? | raw | true | nextnext week Thursday 2:00 PM | 2 |
| events::108::0 | What time does the event start at that location in Portland, OR? | raw | true | 2024-10-16 Wednesday 09:00 | 2 |
| events::109::0 | Which venue description would be suitable for an event with around five thousand attendees? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::110::0 | How long does an activity that lasts for seven days take? | raw | true | nextnext week Thursday 2:00 PM | 2 |
| events::111::0 | Which event location description corresponds to the activity planned for October 14, 2024, at 19:00? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::112::0 | What activity lasts nine days and corresponds to its described location? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::113::0 | Which venue is suitable for an event with two hundred people? | raw | false | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::114::0 | What time does the event take place in Portland, OR? | raw | true | 2024-10-13 Sunday 14:00 | 2 |
| events::115::0 | What time does the event start in Chicago, IL? | raw | false | nextnext week Monday 2:00 PM | 2 |
| events::116::0 | What is an activity with a one-day duration that fits the description of its location? | raw | false | A major cultural and economic center in the southeastern U.S. | 2 |
| events::117::0 | What time is the event for five hundred attendees? | raw | false | nextnext week Thursday 2:00 PM | 2 |
| events::118::0 | Which location description fits the event happening on October 17, 2024, at 14:00? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::119::0 | What time is the event happening in Orlando, FL? | raw | true | next week Sunday 2:00 PM | 2 |
| events::120::0 | What location would work for an event expecting around five hundred people? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::121::0 | What event matches the description of the location for the activity planned for the week after next Friday at 9:00 AM? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::122::0 | What is the timeframe for the activity that lasts six days? | raw | true | next week Sunday 2:00 PM | 2 |
| events::123::0 | What time is the event that can accommodate four thousand people? | raw | true | 2024-10-08 Tuesday 19:00 | 2 |
| events::124::0 | What is the timeframe for the event that lasts five days? | raw | true | next week Sunday 7:00 PM | 2 |
| events::125::0 | What event with a capacity of seven thousand people fits the description of its venue? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::126::0 | What time is the event that expects seven hundred attendees? | raw | true | 2024-10-18 Friday 19:00 | 2 |
| events::127::0 | Which event location description corresponds to the activity planned for October 16, 2024, at 9:00 AM? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::128::0 | What time is the event that has nine hundred people? | raw | true | next week Sunday 7:00 PM | 2 |
| events::129::0 | What time is the event taking place in Los Angeles, CA? | raw | true | 2024-10-26 Saturday 14:00 | 2 |
| events::130::0 | What eight-day activity matches the description of its location? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::131::0 | What time is the event that has a scale of three hundred people? | raw | true | next week Friday 9:00 AM | 2 |
| events::132::0 | What event is happening next Saturday at 2:00 PM, and where will it take place? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::133::0 | What time is the event expected to accommodate eight hundred people? | raw | false | 2024-10-15 Tuesday 09:00 | 2 |
| events::134::0 | Which venue would be suitable for an event with six hundred attendees? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::135::0 | How long does the activity that lasts three weeks take? | raw | true | next week Monday 2:00 PM | 2 |
| events::136::0 | What time is the event that will host eight hundred people? | raw | true | 2024-10-17 Thursday 14:00 | 2 |
| events::137::0 | What activity has a duration of six weeks that matches its location description? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::138::0 | How long does the activity that lasts seven days take? | raw | true | next week Saturday 7:00 PM | 2 |
| events::139::0 | What time is the event taking place in Miami, FL? | raw | true | 2024-10-08 Tuesday 09:00 | 2 |
| events::140::0 | What time does the event start in Seattle, WA? | raw | true |  week Friday 2:00 PM | 2 |
| events::141::0 | At what time is the event that will have nine hundred people? | raw | true | nextnext week Wednesday 9:00 AM | 2 |
| events::142::0 | Which activity that lasts three weeks fits the description of its location? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::143::0 | What time is the event happening in Austin, TX? | raw | false | 2024-10-11 Friday 14:00 | 2 |
| events::144::0 | What type of event venue would be suitable for an activity with a capacity of five hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::145::0 | What five-day activity corresponds with the description of its location? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::146::0 | What time marks the start of an event that lasts for four days? | raw | true | 2024-10-18 Friday 19:00 | 2 |
| events::147::0 | What is the schedule for the event that lasts three days? | raw | true | next week Sunday 9:00 AM | 2 |
| events::148::0 | What activity lasts seven weeks and matches the description of the event location? | raw | false | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::149::0 | What time is the event that will have eight hundred people attending? | raw | false | 2024-10-14 Monday 14:00 | 2 |
| events::150::0 | What time does the event in Austin, TX start? | raw | false | 2024-10-16 Wednesday 09:00 | 2 |
| events::151::0 | What time is the event scheduled for that will have around four hundred people? | raw | true | nextnext week Wednesday 2:00 PM | 2 |
| events::152::0 | What is the timeframe for an activity that lasts one week? | raw | true | 2024-10-16 Wednesday 09:00 | 3 |
| events::153::0 | What event location description corresponds to the activity planned for October 14, 2024, at 9:00 AM? | raw | true | Known for its history, education, and sports teams. | 4 |
| events::154::0 | What time is the event that will host eight hundred people? | raw | true | next week Saturday 7:00 PM | 4 |
| events::155::0 | Which venue would be suitable for hosting an event with around five hundred people? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 3 |
| events::156::0 | What time will the event take place in Portland, OR? | raw | true | 2024-10-17 Thursday 14:00 | 2 |
| events::157::0 | What time is the event that will host eight hundred people? | raw | true | 2024-10-08 Tuesday 19:00 | 2 |
| events::158::0 | Which event location would be suitable for an activity designed for one hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::159::0 | Which two-day activity aligns with its described location? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::160::0 | What time is the event that's expected to have seven hundred people? | raw | true | 2024-10-20 Sunday 09:00 | 2 |
| events::161::0 | What time is the event happening in Washington, DC? | raw | false | nextnext week Thursday 2:00 PM | 2 |
| events::162::0 | Which venue description is suitable for an event with three hundred attendees? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::163::0 | What venue would be suitable for an event with a scale of four hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::164::0 | Which venue would be suitable for an event expecting around two hundred people? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::165::0 | What is the timeline for the activity that lasts three weeks? | raw | true | 2024-10-08 Tuesday 19:00 | 2 |
| events::166::0 | Which event corresponds to the location for the activity planned for the week after next Saturday at 2:00 PM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::167::0 | How long does an activity that lasts four weeks take? | raw | true | 2024-10-18 Friday 19:00 | 2 |
| events::168::0 | What time is the event scheduled to take place at that location in Washington, DC? | raw | true | nextnext week Tuesday 7:00 PM | 2 |
| events::169::0 | What is the activity location description for the event set on October 14, 2024, at 9:00 AM? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::170::0 | What time is the event taking place in San Francisco, CA? | raw | true | 2024-10-13 Sunday 19:00 | 2 |
| events::171::0 | What time is the event happening in Denver, CO? | raw | false | next week Saturday 2:00 PM | 2 |
| events::172::0 | What time will the event take place in Los Angeles, CA? | raw | true | 2024-10-14 Monday 19:00 | 2 |
| events::173::0 | What time is the event taking place in Chicago, IL? | raw | true | 2024-10-08 Tuesday 09:00 | 2 |
| events::174::0 | Which three-day event corresponds with the description of its location? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::175::0 | Which venue would be suitable for an event with nine hundred attendees? | raw | false | Known for its theme parks, including Walt Disney World. | 3 |
| events::176::0 | What time is the event that involves three hundred people? | raw | false | next week Friday 2:00 PM | 3 |
| events::177::0 | What two-week activity aligns with the description of the activity location? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::178::0 | Which event location fits the activity planned for October 17, 2024, at 9:00? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::179::0 | Which venue fits the description for an event accommodating around five hundred people? | raw | false | Known for its history, education, and sports teams. | 2 |
| events::180::0 | Which venue would be suitable for an event with seven hundred attendees? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::181::0 | What time is the event taking place in Austin, TX? | raw | false | nextnext week Tuesday 7:00 PM | 2 |
| events::182::0 | What time is the event that will have four hundred people attending? | raw | true | 2024-10-10 Thursday 19:00 | 2 |
| events::183::0 | Which venue would be suitable for an event with nine hundred attendees? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::184::0 | What three-day event corresponds with the description of its location? | raw | false | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::185::0 | How long does the activity that lasts for three weeks take? | raw | true | 2024-10-24 Thursday 19:00 | 2 |
| events::186::0 | What is the event location description for the activity planned on 2024-10-13 at 19:00? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::187::0 | When does the event that lasts three weeks start? | raw | true | 2024-10-17 Thursday 09:00 | 3 |
| events::188::0 | Which event location description corresponds to the event planned for October 12, 2024, at 9:00? | raw | true | A major business and cultural hub in Texas, known for its skyline. | 2 |
| events::189::0 | What time does the event start in New York, NY? | raw | true | next week Sunday 7:00 PM | 2 |
| events::190::0 | What venue fits the description of an event with a scale of one thousand people? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::191::0 | What activity lasts seven days and fits the description of its location? | raw | false | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::192::0 | Which event location description corresponds to the activity planned for October 12, 2024, at 19:00? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::193::0 | What is the timeframe for the activity that lasts five weeks? | raw | true | next week Sunday 9:00 AM | 2 |
| events::194::0 | What time is the event taking place in New York, NY? | raw | true | next week Saturday 9:00 AM | 2 |
| events::195::0 | What time is the event happening in Philadelphia, PA? | raw | false | 2024-10-18 Friday 09:00 | 2 |
| events::196::0 | What event lasts for nine days and fits the description of its activity location? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::197::0 | How long does the activity that lasts seven days take? | raw | true | 2024-10-18 Friday 14:00 | 3 |
| events::198::0 | What time is the event taking place in Orlando, FL? | raw | false | 2024-10-10 Thursday 14:00 | 2 |
| events::199::0 | Which six-week activity aligns perfectly with its location? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::200::0 | What time does the event start at that location in Portland, OR? | raw | false | next week Saturday 7:00 PM | 2 |
| events::201::0 | How long is the activity that lasts for five weeks? | raw | true | next week Saturday 7:00 PM | 2 |
| events::202::0 | Which event description aligns with the location for the activity planned on October 13, 2024, at 9:00? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::203::0 | How long is the activity that lasts for four weeks? | raw | true | next week Friday 7:00 PM | 2 |
| events::204::0 | Which venue would be suitable for an event accommodating nine hundred people? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::205::0 | What time is the event that will have four hundred attendees? | raw | true | next week Friday 7:00 PM | 2 |
| events::206::0 | What time is the event that will accommodate five hundred people? | raw | true | 2024-10-07 Monday 09:00 | 2 |
| events::207::0 | What time is the event expected to start that will have around three hundred people in attendance? | raw | true | next week Saturday 7:00 PM | 2 |
| events::208::0 | What six-week activity fits the description of its location? | raw | false | A major cultural and economic center in the southeastern U.S. | 2 |
| events::209::0 | What type of venue would work well for an event with around two hundred people? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::210::0 | Which venue is suitable for an event that accommodates five hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::211::0 | What event lasts for six days and fits the description of its location? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::212::0 | What is the timeframe for an activity that lasts for seven weeks? | raw | true | 2024-10-17 Thursday 14:00 | 2 |
| events::213::0 | What event description corresponds to the location for the activity set to take place on October 17, 2024, at 2:00 PM? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::214::0 | What time does the event start in Los Angeles, CA? | raw | true | next week Saturday 9:00 AM | 2 |
| events::215::0 | Which venue is suitable for an event with about one hundred people? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::216::0 | Which event location corresponds to the activity planned for next Saturday at 2:00 PM? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::217::0 | Which venue would be suitable for an event with around four hundred attendees? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::218::0 | What time does the event in Chicago, IL start? | raw | true | 2024-10-20 Sunday 19:00 | 2 |
| events::219::0 | What time is the event happening in Portland, OR? | raw | true | next week Friday 2:00 PM | 2 |
| events::220::0 | What time does an event that lasts for two days start? | raw | true | next week Monday 9:00 AM | 2 |
| events::221::0 | Which event's location description aligns with the activity planned for next Wednesday at 9:00 AM? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::222::0 | Which venue would be suitable for an event scaled for seven hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 3 |
| events::223::0 | When does the activity that lasts for three weeks take place? | raw | true | 2024-10-07 Monday 19:00 | 2 |
| events::224::0 | Which event corresponds to the location description for the activity planned for the week after next Friday at 9:00 AM? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::225::0 | Which venue fits the description for the activity that accommodates one hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::226::0 | Which location is hosting the event that’s taking place the week after next Sunday at 7:00 PM? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::227::0 | Which venue is suitable for an event that accommodates around seven hundred people? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::228::0 | What time does the event take place at that location in Washington, DC? | raw | true | nextnext week Monday 9:00 AM | 2 |
| events::229::0 | At what time is the event taking place in Seattle, WA? | raw | true | 2024-10-18 Friday 19:00 | 2 |
| events::230::0 | What is the event location description for the activity scheduled on October 12, 2024, at 14:00? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::231::0 | What event location would be suitable for an activity with a scale of one hundred people? | raw | false | Known for its theme parks, including Walt Disney World. | 2 |
| events::232::0 | How long is the activity that lasts for eight weeks? | raw | true | nextnext week Thursday 7:00 PM | 2 |
| events::233::0 | What time is the event that will have seven hundred people attending? | raw | true | 2024-10-10 Thursday 19:00 | 2 |
| events::234::0 | What two-day event corresponds to the description of its location? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::235::0 | Which event location corresponds to the activity planned for 19:00 on October 13, 2024? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::236::0 | What type of venue would be suitable for an event with around five hundred attendees? | raw | false | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::237::0 | Which venue would be suitable for an event with about three hundred people? | raw | false | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::238::0 | For an event expecting around three hundred people, what kind of location would be the best fit? | raw | false | The capital of Arizona, known for its hot desert climate. | 2 |
| events::239::0 | Which venue fits the description for the event that accommodates seven hundred people? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::240::0 | What six-week activity fits the description of its location? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::241::0 | Which event corresponds to the location of the activity planned for the week after next Monday at 2:00 PM? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::242::0 | What time is the event that will have six hundred people attending? | raw | false | next week Friday 2:00 PM | 3 |
| events::243::0 | Which event scheduled for October 13, 2024, at 9:00 matches the description of its location? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 3 |
| events::244::0 | What time is the event with four hundred people? | raw | true | next week Saturday 7:00 PM | 2 |
| events::245::0 | What is the scheduled time for the event that accommodates two hundred people? | raw | true | 2024-10-20 Sunday 19:00 | 2 |
| events::246::0 | What event lasts for nine days and has a location that matches its description? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::247::0 | How long will the activity that lasts for eight days take? | raw | true | next week Sunday 9:00 AM | 2 |
| events::248::0 | Which three-week activity aligns with the description of its location? | raw | false | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::249::0 | Is there an event that lasts for seven days and has a description that fits its location? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::250::0 | Which description of the event location corresponds to the activity planned for October 12, 2024, at 9:00? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::251::0 | Which location corresponds to the event with nine hundred attendees? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::252::0 | Which event location corresponds to the activity planned for the week after next Tuesday at 9:00 AM? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::253::0 | What time is the event that is expected to have six hundred people? | raw | true | next week Sunday 2:00 PM | 2 |
| events::254::0 | What time is the event taking place in New York, NY? | raw | true | 2024-10-11 Friday 14:00 | 2 |
| events::255::0 | What time is the event taking place in Orlando, FL? | raw | false | 2024-10-20 Sunday 09:00 | 2 |
| events::256::0 | What event location description fits the activity planned for 2024-10-14 at 9:00? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::257::0 | What time is the event happening in New York, NY? | raw | true | 2024-10-07 Monday 19:00 | 2 |
| events::258::0 | What time is the event that is expected to have two hundred people? | raw | true | nextnext week Wednesday 9:00 AM | 2 |
| events::259::0 | How long does an activity that lasts eight weeks take? | raw | true | 2024-10-07 Monday 14:00 | 2 |
| events::260::0 | What kind of venue would be suitable for an event that expects around three thousand attendees? | raw | false | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::261::0 | Which location description suits the event that was attended by eight hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::262::0 | What time is the event scheduled for with three hundred people attending? | raw | false | nextnext week Tuesday 9:00 AM | 2 |
| events::263::0 | Which event location corresponds to the activity planned for the week after next Friday at 7:00 PM? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::264::0 | What time is the event scheduled for, considering it has a scale of four hundred people? | raw | true | nextnext week Tuesday 2:00 PM | 2 |
| events::265::0 | What time is the event that will host five hundred people? | raw | false | 2024-10-09 Wednesday 09:00 | 2 |
| events::266::0 | What three-week activity fits the description of its location? | raw | false | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::267::0 | Which event corresponds to the location description for the activity scheduled for the week after next Monday at 2:00 PM? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::268::0 | What time is the event happening at that location in Washington, DC? | raw | true | 2024-10-10 Thursday 19:00 | 2 |
| events::269::0 | What venue would be suitable for an event accommodating eight hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::270::0 | What time is the event happening in San Francisco, CA? | raw | false | 2024-10-14 Monday 14:00 | 2 |
| events::271::0 | What four-day event can be identified by its location? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::272::0 | What time is the event taking place in Boston, MA? | raw | true | 2024-10-07 Monday 09:00 | 2 |
| events::273::0 | What time is the event that involves two hundred people? | raw | false | next week Saturday 2:00 PM | 2 |
| events::274::0 | Which venue fits the description of an event that can accommodate eight hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::275::0 | What time does the event take place in Washington, DC? | raw | false | 2024-10-13 Sunday 14:00 | 2 |
| events::276::0 | What is the event location for the activity scheduled next week on Friday at 9:00 AM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::277::0 | What time is the event for eight hundred people? | raw | false | nextnext week Wednesday 7:00 PM | 2 |
| events::278::0 | What time is the event for eight hundred people? | raw | false | 2024-10-13 Sunday 19:00 | 2 |
| events::279::0 | At what time does the activity that lasts for one day take place? | raw | true | 2024-10-08 Tuesday 14:00 | 2 |
| events::280::0 | Is there an activity that lasts one day and matches the description of its location? | raw | false | Known for its beautiful beaches and mild climate. | 2 |
| events::281::0 | What is the event location for the scheduled event on October 16, 2024, at 7:00 PM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::282::0 | Which location description corresponds to the event planned for October 14, 2024, at 9:00? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::283::0 | How much time does an activity that lasts five weeks take? | raw | true | 2024-10-13 Sunday 09:00 | 3 |
| events::284::0 | What time is the event happening in Chicago, IL? | raw | false | 2024-10-20 Sunday 14:00 | 4 |
| events::285::0 | What venue would be suitable for an event that accommodates five hundred people? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 3 |
| events::286::0 | Can someone name an event that lasts for seven days and fits its described location? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::287::0 | What is the schedule for the nine-day activity? | raw | true | nextnext week Wednesday 9:00 AM | 2 |
| events::288::0 | Which venue would be suitable for an event that accommodates eight hundred people? | raw | true | The heart of Silicon Valley, known for its tech industry. | 2 |
| events::289::0 | What five-day event matches the description of its location? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::290::0 | What time is the event that will have a hundred people? | raw | false | nextnext week Wednesday 2:00 PM | 2 |
| events::291::0 | What activity that lasts four weeks aligns with the description of its location? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::292::0 | Is there an event that lasts two days and fits its location description? | raw | true | The capital of Arizona, known for its hot desert climate. | 2 |
| events::293::0 | What time is the event taking place in Miami, FL? | raw | true | nextnext week Monday 9:00 AM | 2 |
| events::294::0 | What three-week activity aligns with the description of its location? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::295::0 | What event fits the location description for the activity planned for next Sunday at 7:00 PM? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::296::0 | What time does the event start in Atlanta, GA? | raw | false | 2024-10-18 Friday 14:00 | 2 |
| events::297::0 | What time is the event that expects a crowd of six thousand people? | raw | true | next week Friday 9:00 AM | 2 |
| events::298::0 | Which venue corresponds to the event designed for six hundred people? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::299::0 | What time is the event happening in San Francisco, CA? | raw | true | 2024-10-18 Friday 09:00 | 2 |
| events::300::0 | What six-week activity matches the description of its location? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::301::0 | When does the activity that lasts for three weeks take place? | raw | true | nextnext week Tuesday 9:00 AM | 2 |
| events::302::0 | When does the activity that lasts one week take place? | raw | true | next week Friday 7:00 PM | 2 |
| events::303::0 | What time does the event start in Portland, OR? | raw | true | nextnext week Monday 2:00 PM | 2 |
| events::304::0 | Which venue would be suitable for an event that can accommodate four hundred people? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::305::0 | Which event location description would be suitable for accommodating two hundred people? | raw | true | The capital of Arizona, known for its hot desert climate. | 2 |
| events::306::0 | What activity lasts six weeks and matches the description of its location? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 4 |
| events::307::0 | What time is the event for three hundred people? | raw | false | 2024-10-20 Sunday 09:00 | 3 |
| events::308::0 | What kind of venue would be suitable for an event with about seven hundred people? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 4 |
| events::309::0 | What time is the event that will have about two hundred people attending? | raw | false | next week Sunday 2:00 PM | 3 |
| events::310::0 | Which venue description corresponds to the activity planned for the week after next Friday at 2:00 PM? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::311::0 | What time does the event start in Seattle, WA? | raw | true | next week Saturday 2:00 PM | 3 |
| events::312::0 | What time is the event for four hundred people being held? | raw | true | next week Friday 2:00 PM | 4 |
| events::313::0 | Which event takes place at the location for the activity on the week after next Tuesday at 7:00 PM? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::314::0 | Which venue would be suitable for an event hosting around two hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 3 |
| events::315::0 | Which event location would be suitable for an activity designed for five hundred people? | raw | false | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::316::0 | What time does the event start at that location in Denver, CO? | raw | false | nextnext week Monday 9:00 AM | 2 |
| events::317::0 | What time is the event expected to start that will have nine hundred people attending? | raw | true | nextnext week Wednesday 2:00 PM | 2 |
| events::318::0 | What activity lasts eight days and aligns with its location description? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::319::0 | What venue would be suitable for an event with around six hundred people? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 3 |
| events::320::0 | What time is the event that involves six hundred people? | raw | true | 2024-10-16 Wednesday 09:00 | 3 |
| events::321::0 | Which event location matches the activity scheduled for next week on Monday at 7:00 PM? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::322::0 | What time is the event happening in Los Angeles, CA? | raw | false | 2024-10-09 Wednesday 09:00 | 2 |
| events::323::0 | What event corresponds to the location for the activity planned for the week after next Tuesday at 2:00 PM? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::324::0 | What kind of venue would be suitable for hosting an event for two hundred people? | raw | false | Known for its history, education, and sports teams. | 2 |
| events::325::0 | What week-long activity matches the description of its location? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::326::0 | Which event location would be suitable for an activity with around one hundred people? | raw | false | The capital of the U.S., known for its national monuments and museums. | 3 |
| events::327::0 | What time is the event expected to take place that will have around four thousand attendees? | raw | false | nextnext week Monday 2:00 PM | 3 |
| events::328::0 | What time is the event happening in Houston, TX? | raw | true | next week Saturday 9:00 AM | 3 |
| events::329::0 | What is the time for an event that lasts for two days? | raw | false | 2024-10-19 Saturday 19:00 | 2 |
| events::330::0 | Which venue would be suitable for an event with around three hundred people? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 3 |
| events::331::0 | What are the scheduled times for the event that lasts two days? | raw | true | nextnext week Tuesday 9:00 AM | 2 |
| events::332::0 | What four-day event corresponds with the description of its location? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::333::0 | When does the activity that lasts for two weeks take place? | raw | true | 2024-10-17 Thursday 09:00 | 2 |
| events::334::0 | What time is the event that will have three hundred people attending? | raw | false | nextnext week Wednesday 2:00 PM | 2 |
| events::335::0 | What time will the event take place in Boston, MA? | raw | true | 2024-10-20 Sunday 09:00 | 2 |
| events::336::0 | What time is the event taking place in Miami, FL? | raw | true | 2024-10-12 Saturday 09:00 | 2 |
| events::337::0 | What is the timeframe for an event that lasts three days? | raw | true | 2024-10-20 Sunday 19:00 | 3 |
| events::338::0 | Which location coincides with the event scheduled for 9:00 AM on the Saturday after next? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::339::0 | Which event corresponds to the location details for the activity planned on October 12, 2024, at 19:00? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::340::0 | What time is the event taking place in Austin, TX? | raw | true | 2024-10-11 Friday 09:00 | 2 |
| events::341::0 | What is the time for an event that lasts one day? | raw | true | 2024-10-08 Tuesday 14:00 | 2 |
| events::342::0 | Which venue fits the description for an event that can accommodate seven hundred people? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::343::0 | What time is the event that has a scale of six hundred people? | raw | true | 2024-10-18 Friday 14:00 | 2 |
| events::344::0 | What three-week-long event corresponds with its described location? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::345::0 | What time does the event take place in Seattle, WA? | raw | false | 2024-10-11 Friday 09:00 | 2 |
| events::346::0 | Which location description fits the event scheduled for October 12, 2024, at 9:00 AM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::347::0 | What seven-day event matches the description of its location? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::348::0 | Which activity that lasts four days fits the description of its location? | raw | true | The heart of Silicon Valley, known for its tech industry. | 2 |
| events::349::0 | What is the event location for the activity planned on October 17, 2024, at 14:00? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::350::0 | How long does the activity that lasts four weeks take? | raw | true | next week Monday 9:00 AM | 2 |
| events::351::0 | What time will the event take place in Washington, DC? | raw | false | nextnext week Thursday 9:00 AM | 2 |
| events::352::0 | What time does the event that lasts four days start? | raw | false | next week Sunday 2:00 PM | 2 |
| events::353::0 | What time does the event that lasts for two days start? | raw | false | next week Friday 9:00 AM | 2 |
| events::354::0 | Which venue would be suitable for an event with around five hundred attendees? | raw | false | A major cultural and economic center in the southeastern U.S. | 3 |
| events::355::0 | What time is the event happening at that location in San Francisco, CA? | raw | true | next week Friday 2:00 PM | 2 |
| events::356::0 | What time is the event that will have seven hundred attendees? | raw | true | next week Thursday 2:00 PM | 2 |
| events::357::0 | What time is the event that will accommodate seven hundred people? | raw | true | 2024-10-19 Saturday 09:00 | 2 |
| events::358::0 | What two-week event fits the description of its location? | raw | false | Known for its history, education, and sports teams. | 2 |
| events::359::0 | Is there an event that lasts for eight days and has a name that reflects its location? | raw | true | A major business and cultural hub in Texas, known for its skyline. | 2 |
| events::360::0 | How long is an activity that lasts for one day? | raw | true | nextnext week Thursday 7:00 PM | 2 |
| events::361::0 | What time is the four-day activity scheduled to start? | raw | false | nextnext week Thursday 2:00 PM | 2 |
| events::362::0 | What time is the event taking place in Houston, TX? | raw | true | 2024-10-20 Sunday 09:00 | 3 |
| events::363::0 | What time is the event in Boston, MA? | raw | false | nextnext week Monday 9:00 AM | 2 |
| events::364::0 | Which event is associated with the location for the activity planned on 2024-10-12 at 9:00? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::365::0 | What time is the event taking place in San Francisco, CA? | raw | true | 2024-10-20 Sunday 09:00 | 2 |
| events::366::0 | What time is the event that has nine hundred people? | raw | true | nextnext week Wednesday 9:00 AM | 2 |
| events::367::0 | Which venue would be suitable for an event with about three hundred attendees? | raw | false | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::368::0 | What activity lasts six days and matches the description of its location? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| events::369::0 | What time will the event with nine hundred people take place? | raw | false | 2024-10-16 Wednesday 14:00 | 3 |
| events::370::0 | What time is the event happening in Washington, DC? | raw | true | next week Wednesday 7:00 PM | 2 |
| events::371::0 | What time does the event start in Atlanta, GA? | raw | true | 2024-10-14 Monday 09:00 | 2 |
| events::372::0 | Which event location corresponds to the activity planned for the week after next Monday at 2:00 PM? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 2 |
| events::373::0 | What time does the event lasting nine days start? | raw | false | nextnext week Thursday 9:00 AM | 2 |
| events::374::0 | Which venue is suitable for an event with around nine hundred people? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::375::0 | Is there an event that lasts for seven days and corresponds with its location's description? | raw | false | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::376::0 | When does the two-week activity take place? | raw | false | 2024-10-20 Sunday 19:00 | 2 |
| events::377::0 | Which venue description matches the activity planned for October 15, 2024, at 9:00? | raw | true | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::378::0 | What time is the event taking place at that location in Las Vegas, NV? | raw | true | nextnext week Tuesday 2:00 PM | 2 |
| events::379::0 | What time is the event expected to start with a scale of six hundred people? | raw | true | 2024-10-10 Thursday 14:00 | 2 |
| events::380::0 | What two-week-long activity fits the description of its location? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::381::0 | What time is the event that will have seven hundred people? | raw | true | 2024-10-11 Friday 19:00 | 2 |
| events::382::0 | What one-week activity corresponds with its location? | raw | false | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::383::0 | Which event location would be suitable for an activity with two hundred people? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::384::0 | What seven-day activity fits the description of its location? | raw | false | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::385::0 | When does the event that lasts six months take place? | raw | true | 2024-10-14 Monday 19:00 | 2 |
| events::386::0 | What time does the event that lasts for three days start? | raw | true | 2024-10-10 Thursday 09:00 | 2 |
| events::387::0 | What time is the event expected to have around five thousand people? | raw | true | nextnext week Wednesday 2:00 PM | 2 |
| events::388::0 | Which location is set to host the event scheduled for the week after next Monday at 2:00 PM? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 3 |
| events::389::0 | What kind of event location would be suitable for an activity with around eight thousand people? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::390::0 | What kind of venue would be suitable for an event with four hundred people? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::391::0 | How long is the activity that lasts for nine days? | raw | true | next week Saturday 7:00 PM | 2 |
| events::392::0 | What venue would be suitable for an event with nine hundred attendees? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::393::0 | What time is the event happening at the location in Miami, FL? | raw | false | nextnext week Wednesday 2:00 PM | 3 |
| events::394::0 | Is there a three-day event that fits the description of its location? | raw | false | Known for its history, education, and sports teams. | 3 |
| events::395::0 | What is the time for the event that will host four hundred people? | raw | false | 2024-10-09 Wednesday 09:00 | 2 |
| events::396::0 | Which event venue would be suitable for an activity with nine hundred attendees? | raw | false | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::397::0 | What event is happening on October 17, 2024, at 19:00 that matches the description of its venue? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 3 |
| events::398::0 | How long does the activity that lasts eight weeks take? | raw | true | next week Sunday 7:00 PM | 2 |
| events::399::0 | How long will an activity that lasts nine weeks take? | raw | true | nextnext week Wednesday 7:00 PM | 2 |
| events::400::0 | Which one-day event corresponds to the description of its location? | raw | false | Famous for its entertainment, casinos, and vibrant nightlife. | 4 |
| events::401::0 | What location corresponds to the event planned for next Monday at 9:00 AM? | raw | true | The capital of the U.S., known for its national monuments and museums. | 4 |
| events::402::0 | What time is the event that will have two hundred people in attendance? | raw | true | 2024-10-08 Tuesday 14:00 | 4 |
| events::403::0 | What time does the event start at that location in Los Angeles, CA? | raw | false | 2024-10-10 Thursday 19:00 | 4 |
| events::404::0 | What event location description corresponds to the activity scheduled for October 12, 2024, at 9:00? | raw | true | The capital of Texas, known for its music scene and cultural events. | 4 |
| events::405::0 | What time does the event in Washington, DC start? | raw | false | nextnext week Thursday 7:00 PM | 4 |
| events::406::0 | How long will the activity that lasts seven days take? | raw | true | 2024-10-07 Monday 09:00 | 4 |
| events::407::0 | What is the event location description that corresponds to the activity planned for 9:00 on October 14, 2024? | raw | true | A major cultural and economic center in the southeastern U.S. | 2 |
| events::408::0 | What is the date for the event that lasts three weeks? | raw | true | next week Saturday 2:00 PM | 2 |
| events::409::0 | What activity has a duration of nine weeks based on its location description? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::410::0 | What event location description matches the activity planned for October 15, 2024, at 9:00? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::411::0 | Which type of event location would be suitable for an activity involving five hundred people? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::412::0 | What event lasts four days and fits the description of its location? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::413::0 | What’s a venue that can accommodate an event with around six thousand people? | raw | true | A major cultural and economic center in the southeastern U.S. | 3 |
| events::414::0 | What time is the event that involves four hundred people? | raw | true | next week Saturday 9:00 AM | 3 |
| events::415::0 | What is the time for an event that lasts for nine days? | raw | true | 2024-10-16 Wednesday 09:00 | 3 |
| events::416::0 | Which location will host the event taking place the week after next Wednesday at 2:00 PM? | raw | true | Famous for the Alamo and its rich Texan culture. | 3 |
| events::417::0 | What activity lasts for six weeks and fits the description of its location? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 3 |
| events::418::0 | What time is the event taking place in Portland, OR? | raw | false | nextnext week Wednesday 9:00 AM | 2 |
| events::419::0 | What time does the event take place at the location in Atlanta, GA? | raw | true | next week Wednesday 7:00 PM | 2 |
| events::420::0 | What is the time for an event that lasts two days? | raw | true | 2024-10-19 Saturday 19:00 | 2 |
| events::421::0 | Which event location corresponds to the activity scheduled for 2:00 PM on the weekend after next Saturday? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::422::0 | What is the scheduled time for the event that accommodates one hundred people? | raw | false | 2024-10-12 Saturday 19:00 | 2 |
| events::423::0 | What is the timeframe for an event that lasts nine days? | raw | true | 2024-10-10 Thursday 14:00 | 2 |
| events::424::0 | What time is the event happening in Washington, DC? | raw | false | nextnext week Thursday 2:00 PM | 2 |
| events::425::0 | Which activity location is associated with the event taking place on October 17, 2024, at 19:00? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::426::0 | How long does an activity that lasts eight weeks take? | raw | true | 2024-10-08 Tuesday 19:00 | 2 |
| events::427::0 | Which description of the event location fits the activity planned for October 13, 2024, at 9:00 AM? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::428::0 | Which six-day activity fits the description of its location? | raw | false | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::429::0 | What kind of event location would be suitable for an activity with about one hundred people? | raw | false | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::430::0 | What time is the event planned for five hundred people? | raw | false | 2024-10-09 Wednesday 19:00 | 3 |
| events::431::0 | Which event location description fits the activity planned for October 13, 2024, at 9:00? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::432::0 | What is the schedule for the activity that lasts nine weeks? | raw | true | 2024-10-14 Monday 19:00 | 2 |
| events::433::0 | What event location corresponds to the activity planned for the week after next Sunday at 2:00 PM? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 3 |
| events::434::0 | Which event location matches the description of the event scheduled for October 16, 2024, at 7:00 PM? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::435::0 | Which event location description corresponds to the activity planned for October 17, 2024, at 14:00? | raw | true | The capital of Texas, known for its music scene and cultural events. | 3 |
| events::436::0 | Which event designed for nine hundred people aligns with the description of its venue? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::437::0 | Which event venue would be suitable for an activity involving around two hundred people? | raw | false | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::438::0 | What time is the event that is expected to have eight hundred attendees? | raw | true | 2024-10-16 Wednesday 19:00 | 2 |
| events::439::0 | What event lasts for seven days and matches the description of its location? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::440::0 | What time is the event that has a scale of seven hundred people? | raw | true | 2024-10-19 Saturday 19:00 | 2 |
| events::441::0 | What time is the event expected to start for a gathering of three hundred people? | raw | true | 2024-10-20 Sunday 09:00 | 2 |
| events::442::0 | Which location is set for the event happening at 7:00 PM the week after next Wednesday? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::443::0 | Is there an event that lasts seven days that fits the description of its location? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::444::0 | What venue would be suitable for an event with around five hundred people? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::445::0 | What description of the activity location corresponds to the event on October 13, 2024, at 19:00? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 2 |
| events::446::0 | What time is the event scheduled for four hundred people? | raw | false | next week Saturday 2:00 PM | 3 |
| events::447::0 | Which venue fits the description for the event that can accommodate three hundred people? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 3 |
| events::448::0 | What time is the event taking place in New York, NY? | raw | false | 2024-10-13 Sunday 19:00 | 2 |
| events::449::0 | What event lasts four days and matches the description of its location? | raw | false | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::450::0 | What kind of event location would be suitable for an activity with around four hundred people? | raw | false | The capital of the U.S., known for its national monuments and museums. | 2 |
| events::451::0 | When does the activity that lasts eight weeks take place? | raw | true | nextnext week Wednesday 7:00 PM | 2 |
| events::452::0 | Which venue fits the description of an event accommodating eight hundred people? | raw | false | Famous for its entertainment, casinos, and vibrant nightlife. | 2 |
| events::453::0 | What time is the one-day activity scheduled for? | raw | false | nextnext week Monday 9:00 AM | 2 |
| events::454::0 | What event location matches the activity scheduled for October 16, 2024, at 9:00 AM? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::455::0 | What venue would be suitable for an event that accommodates eight hundred people? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::456::0 | What seven-day event fits the description of its location? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 2 |
| events::457::0 | What time does the one-day activity take place? | raw | false | next week Friday 2:00 PM | 2 |
| events::458::0 | What time is the event scheduled for at that location in Denver, CO? | raw | false | 2024-10-08 Tuesday 09:00 | 2 |
| events::459::0 | What time does the event take place in Atlanta, GA? | raw | true | 2024-10-16 Wednesday 09:00 | 2 |
| events::460::0 | Which venue can accommodate six hundred people for the event? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 3 |
| events::461::0 | What eight-day activity fits the description of its location? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 4 |
| events::462::0 | What time does the event start in Miami, FL? | raw | false | nextnext week Wednesday 7:00 PM | 4 |
| events::463::0 | Which location would be suitable for an event with around seven hundred attendees? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 2 |
| events::464::0 | Which venue corresponds to the event planned for next Monday at 9:00 AM? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::465::0 | What is the time frame for an activity that lasts one day? | raw | false | 2024-10-18 Friday 14:00 | 2 |
| events::466::0 | Which event location fits the description for the activity planned for next week on Friday at 7:00 PM? | raw | true | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::467::0 | What time does the event that lasts six days start? | raw | true | 2024-10-17 Thursday 14:00 | 2 |
| events::468::0 | What time is the event taking place in New York, NY? | raw | true | 2024-10-18 Friday 14:00 | 2 |
| events::469::0 | When does the activity that lasts four weeks take place? | raw | true | 2024-10-11 Friday 14:00 | 2 |
| events::470::0 | What time does the event take place in Miami, FL? | raw | true | 2024-10-20 Sunday 14:00 | 2 |
| events::471::0 | What time is the event that is expecting five hundred people? | raw | true | nextnext week Wednesday 2:00 PM | 2 |
| events::472::0 | Which event location description corresponds to the activity planned for October 15, 2024, at 9:00 AM? | raw | true | Famous for its eco-friendliness and vibrant arts scene. | 2 |
| events::473::0 | How long does the activity that lasts for four weeks take? | raw | true | 2024-10-08 Tuesday 09:00 | 2 |
| events::474::0 | Which event location would be suitable for a gathering of around one hundred people? | raw | true | The capital of Texas, known for its music scene and cultural events. | 3 |
| events::475::0 | Which event location corresponds to the activity planned for October 14, 2024, at 2:00 PM? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 4 |
| events::476::0 | What event lasts for five days and corresponds to the description of its location? | raw | true | Known for its history, education, and sports teams. | 3 |
| events::477::0 | What time is the event that is expected to have eight hundred people? | raw | false | 2024-10-19 Saturday 09:00 | 2 |
| events::478::0 | What time is the event in Portland, OR? | raw | true | 2024-10-07 Monday 14:00 | 2 |
| events::479::0 | What time does the event in Atlanta, GA start? | raw | true | next week Friday 7:00 PM | 2 |
| events::480::0 | Which event location description corresponds to the activity planned for October 12, 2024, at 14:00? | raw | true | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::481::0 | What activity lasting eight days fits the description of its location? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 2 |
| events::482::0 | How long will the activity that lasts for six days take? | raw | true | next week Sunday 7:00 PM | 2 |
| events::483::0 | Which event location corresponds to the activity planned for October 11, 2024, at 7:00 PM? | raw | true | Known for its theme parks, including Walt Disney World. | 2 |
| events::484::0 | Which eight-day activity aligns with the description of its location? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 2 |
| events::485::0 | What is the ending time for an event that lasts for five days? | raw | true | 2024-10-20 Sunday 09:00 | 3 |
| events::486::0 | What type of activity location would be suitable for an event with around two hundred people? | raw | false | A major cultural and economic center in the southeastern U.S. | 2 |
| events::487::0 | Which venue corresponds to the event planned for the week after next Monday at 2:00 PM? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::488::0 | What time does the event that lasts for two days start? | raw | true | next week Sunday 2:00 PM | 2 |
| events::489::0 | Which event location corresponds to the activity that is scheduled for the week after next Monday at 2:00 PM? | raw | true | Known for its history, education, and sports teams. | 2 |
| events::490::0 | What is the time for the event that involves three hundred people? | raw | true | nextnext week Wednesday 7:00 PM | 2 |
| events::491::0 | What is the time allocated for an activity that lasts one day? | raw | false | nextnext week Wednesday 2:00 PM | 2 |
| events::492::0 | Which venue is suitable for an event that accommodates nine hundred people? | raw | true | Known for its beautiful beaches and mild climate. | 2 |
| events::493::0 | Which event location description corresponds to the activity planned for the week after next Tuesday at 2:00 PM? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 2 |
| events::494::0 | What is the time frame for an event that lasts four days? | raw | true | 2024-10-09 Wednesday 14:00 | 2 |
| events::495::0 | What five-day event fits the description of its location? | raw | false | The capital of Texas, known for its music scene and cultural events. | 2 |
| events::496::0 | What time does the five-day event start? | raw | false |  week Friday 7:00 PM | 2 |
| events::497::0 | What time is the event that will have eight hundred people attending? | raw | false | 2024-10-20 Sunday 14:00 | 2 |
| events::498::0 | For an event with eight hundred attendees, which location would be suitable? | raw | false | A major city in Texas, known for its energy industry and space exploration. | 2 |
| events::499::0 | What is the date for the event that lasts five weeks? | raw | true | 2024-10-18 Friday 09:00 | 2 |
| food::0::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake | 1 |
| food::1::0 | What dishes have you recommended to me before? | raw | true | Apple Pie | 1 |
| food::2::0 | What dishes have you recommended to me before? | raw | false | Fruit | 1 |
| food::3::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Chocolate Dipped Bacon | 1 |
| food::4::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Honey Glazed Ham | 1 |
| food::5::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::6::0 | What dishes have you recommended to me before? | raw | true | Banana Bread, Rice Krispies, Honey | 1 |
| food::7::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate | 1 |
| food::8::0 | What dishes have you recommended to me before? | raw | false | Seafood, Mushroom Risotto, Aged Cheddar | 1 |
| food::9::0 | What dishes have you recommended to me before? | raw | false | Banana Bread | 1 |
| food::10::0 | What dishes have you recommended to me before? | raw | false | Seafood, Miso Soup, Ramen | 1 |
| food::11::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Maple Bacon | 1 |
| food::12::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Caramel | 1 |
| food::13::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Maple Bacon | 1 |
| food::14::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Pecan Praline, Salted Lassi | 1 |
| food::15::0 | What dishes have you recommended to me before? | raw | true | Prosciutto and Melon, Salted Peanut Butter Cookies, Salted Caramel | 1 |
| food::16::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream, Chocolate Covered Pretzels, Pecan Praline | 1 |
| food::17::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Maple Ice Cream, Prosciutto and Melon | 1 |
| food::18::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham | 1 |
| food::19::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Chocolate Dipped Bacon | 1 |
| food::20::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon | 1 |
| food::21::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Salted Peanut Butter Cookies | 1 |
| food::22::0 | What dishes have you recommended to me before? | raw | false | Fruit, Chocolate Cake, Honey | 1 |
| food::23::0 | What dishes have you recommended to me before? | raw | false | Jelly | 1 |
| food::24::0 | What dishes have you recommended to me before? | raw | false | Fruit, Banana Bread | 1 |
| food::25::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Sea Salt Chocolate | 1 |
| food::26::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Salted Maple Ice Cream | 1 |
| food::27::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Honey, Jelly, Candy | 1 |
| food::28::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Jelly | 1 |
| food::29::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Salted Maple Ice Cream, Prosciutto and Melon | 1 |
| food::30::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon | 1 |
| food::31::0 | What dishes have you recommended to me before? | raw | true | Sea Salt Chocolate, Chocolate Covered Pretzels, Salted Butterscotch Pudding | 1 |
| food::32::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Salted Butterscotch Pudding, Honey Glazed Ham | 1 |
| food::33::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Apple Pie | 1 |
| food::34::0 | What dishes have you recommended to me before? | raw | false | Mango Sweet and Sour Sauce, Guava Jelly | 1 |
| food::35::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Pecan Praline | 1 |
| food::36::0 | What dishes have you recommended to me before? | raw | false | Jelly, Custard | 1 |
| food::37::0 | What dishes have you recommended to me before? | raw | true | Salted Peanut Butter Cookies, Honey Glazed Ham | 1 |
| food::38::0 | What dishes have you recommended to me before? | raw | false | Anchovy Pizza, Ramen | 1 |
| food::39::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes | 1 |
| food::40::0 | What dishes have you recommended to me before? | raw | false | Tandoori Chicken, Buffalo Wings | 1 |
| food::41::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel | 1 |
| food::42::0 | What dishes have you recommended to me before? | raw | false | Aged Cheddar | 1 |
| food::43::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Salted Maple Ice Cream, Prosciutto and Melon | 1 |
| food::44::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::45::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Salted Peanut Butter Cookies | 1 |
| food::46::0 | What dishes have you recommended to me before? | raw | true | Honey | 1 |
| food::47::0 | What dishes have you recommended to me before? | raw | true | Salted Maple Ice Cream | 1 |
| food::48::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Caramel, Salted Lassi | 1 |
| food::49::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies | 1 |
| food::50::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::51::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Maple Ice Cream, Pecan Praline | 1 |
| food::52::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Sea Salt Chocolate | 1 |
| food::53::0 | What dishes have you recommended to me before? | raw | false | Candy | 1 |
| food::54::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Salted Lassi | 1 |
| food::55::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Fruit | 1 |
| food::56::0 | What dishes have you recommended to me before? | raw | false | Peking Duck | 1 |
| food::57::0 | What dishes have you recommended to me before? | raw | false | Banana Bread, Fruit | 1 |
| food::58::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Maple Syrup Pancakes, Honey | 1 |
| food::59::0 | What dishes have you recommended to me before? | raw | true | Salted Lassi, Salted Butter Toffee | 1 |
| food::60::0 | What dishes have you recommended to me before? | raw | false | Creme Brulee, Panna Cotta, Vanilla Milkshake | 1 |
| food::61::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Butterscotch Pudding | 1 |
| food::62::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Salted Caramel | 1 |
| food::63::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Salted Lassi | 1 |
| food::64::0 | What dishes have you recommended to me before? | raw | false | Baklava, Brownies, Rice Krispies, Fruit | 1 |
| food::65::0 | What dishes have you recommended to me before? | raw | false | Candy, Baklava, Chocolate Cake | 1 |
| food::66::0 | What dishes have you recommended to me before? | raw | false | Mango Lassi, Vanilla Milkshake, Creme Brulee | 1 |
| food::67::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee | 1 |
| food::68::0 | What dishes have you recommended to me before? | raw | true | Sea Salt Chocolate | 1 |
| food::69::0 | What dishes have you recommended to me before? | raw | false | Baklava, Jelly | 1 |
| food::70::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Sea Salt Chocolate, Maple Bacon | 1 |
| food::71::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon | 1 |
| food::72::0 | What dishes have you recommended to me before? | raw | true | Salted Butter Toffee, Maple Bacon, Salted Butterscotch Pudding, Chocolate Dipped Bacon | 1 |
| food::73::0 | What dishes have you recommended to me before? | raw | false | Rice Pudding, Coconut Pudding | 1 |
| food::74::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Caramel | 1 |
| food::75::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Candy, Brownies | 1 |
| food::76::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding | 1 |
| food::77::0 | What dishes have you recommended to me before? | raw | false | Banana Bread, Brownies, Maple Syrup Pancakes | 1 |
| food::78::0 | What dishes have you recommended to me before? | raw | false | Custard | 1 |
| food::79::0 | What dishes have you recommended to me before? | raw | true | Salted Maple Ice Cream | 1 |
| food::80::0 | What dishes have you recommended to me before? | raw | false | Japanese Sweet Egg Tamagoyaki | 1 |
| food::81::0 | What dishes have you recommended to me before? | raw | true | Honey Glazed Ham, Chocolate Covered Pretzels, Salted Maple Ice Cream | 1 |
| food::82::0 | What dishes have you recommended to me before? | raw | false | Apple Pie | 1 |
| food::83::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies | 1 |
| food::84::0 | What dishes have you recommended to me before? | raw | false | Bacon | 1 |
| food::85::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Salted Caramel | 1 |
| food::86::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::87::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Caramel, Salted Lassi | 1 |
| food::88::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Pecan Praline | 1 |
| food::89::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Honey Glazed Ham | 1 |
| food::90::0 | What dishes have you recommended to me before? | raw | false | Spicy Szechuan Tofu | 1 |
| food::91::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi | 1 |
| food::92::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie, Honey | 1 |
| food::93::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream | 1 |
| food::94::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Ramen, Seafood | 1 |
| food::95::0 | What dishes have you recommended to me before? | raw | true | Parmesan Cheese, Nori Seaweed | 1 |
| food::96::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie, Maple Syrup Pancakes | 1 |
| food::97::0 | What dishes have you recommended to me before? | raw | true | Salted Maple Ice Cream, Chocolate Dipped Bacon | 1 |
| food::98::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Salted Peanut Butter Cookies, Honey Glazed Ham, Sea Salt Chocolate | 1 |
| food::99::0 | What dishes have you recommended to me before? | raw | false | Baklava, Custard, Chocolate Cake | 1 |
| food::100::0 | What dishes have you recommended to me before? | raw | false | Fruit | 1 |
| food::101::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Honey Glazed Ham | 1 |
| food::102::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Chocolate Covered Pretzels | 1 |
| food::103::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline | 1 |
| food::104::0 | What dishes have you recommended to me before? | raw | true | Pecan Praline, Prosciutto and Melon | 1 |
| food::105::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Prosciutto and Melon | 1 |
| food::106::0 | What dishes have you recommended to me before? | raw | false | Salsa, Spicy Hotpot | 1 |
| food::107::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Baklava, Honey | 1 |
| food::108::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Pecan Praline | 1 |
| food::109::0 | What dishes have you recommended to me before? | raw | false | Miso Soup | 1 |
| food::110::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Pecan Pie | 1 |
| food::111::0 | What dishes have you recommended to me before? | raw | true | Nori Seaweed | 1 |
| food::112::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream, Salted Butterscotch Pudding, Salted Caramel | 1 |
| food::113::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Prosciutto and Melon | 1 |
| food::114::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Butterscotch Pudding, Prosciutto and Melon | 1 |
| food::115::0 | What dishes have you recommended to me before? | raw | true | Tomato Sauce, Miso Soup | 1 |
| food::116::0 | What dishes have you recommended to me before? | raw | false | Chili, Cajun Shrimp, Spicy Hotpot | 1 |
| food::117::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding | 1 |
| food::118::0 | What dishes have you recommended to me before? | raw | false | Brownies, Candy, Baklava | 1 |
| food::119::0 | What dishes have you recommended to me before? | raw | true | Parmesan Cheese, Miso Soup | 1 |
| food::120::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Chocolate Covered Pretzels | 1 |
| food::121::0 | What dishes have you recommended to me before? | raw | false | Beef Stew, Soy Sauce | 1 |
| food::122::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Honey Glazed Ham | 1 |
| food::123::0 | What dishes have you recommended to me before? | raw | false | Fruit | 1 |
| food::124::0 | What dishes have you recommended to me before? | raw | false | Custard, Chocolate Cake | 1 |
| food::125::0 | What dishes have you recommended to me before? | raw | false | Custard | 1 |
| food::126::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Salted Butterscotch Pudding | 1 |
| food::127::0 | What dishes have you recommended to me before? | raw | false | Parmesan Cheese, Miso Soup | 1 |
| food::128::0 | What dishes have you recommended to me before? | raw | false | BBQ Ribs, Sweet Soy Sauce Dishes | 1 |
| food::129::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Maple Ice Cream | 1 |
| food::130::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham, Salted Butterscotch Pudding | 1 |
| food::131::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding | 1 |
| food::132::0 | What dishes have you recommended to me before? | raw | false | Fruit, Baklava | 1 |
| food::133::0 | What dishes have you recommended to me before? | raw | false | Honey | 1 |
| food::134::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce, Miso Soup | 1 |
| food::135::0 | What dishes have you recommended to me before? | raw | false | Apple Pie, Donuts, Chocolate Cake | 1 |
| food::136::0 | What dishes have you recommended to me before? | raw | true | Prosciutto and Melon | 1 |
| food::137::0 | What dishes have you recommended to me before? | raw | false | Donuts, Maple Syrup Pancakes, Candy | 1 |
| food::138::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding | 1 |
| food::139::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie | 1 |
| food::140::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Honey Glazed Ham, Salted Maple Ice Cream, Chocolate Covered Pretzels | 1 |
| food::141::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Maple Ice Cream, Chocolate Covered Pretzels | 1 |
| food::142::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels | 1 |
| food::143::0 | What dishes have you recommended to me before? | raw | false | Apple Pie | 1 |
| food::144::0 | What dishes have you recommended to me before? | raw | false | Jelly | 1 |
| food::145::0 | What dishes have you recommended to me before? | raw | false | Apple Pie, Candy | 1 |
| food::146::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Chocolate Dipped Bacon, Salted Caramel | 1 |
| food::147::0 | What dishes have you recommended to me before? | raw | false | Tandoori Chicken, Cajun Shrimp | 1 |
| food::148::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Chocolate Dipped Bacon | 1 |
| food::149::0 | What dishes have you recommended to me before? | raw | false | Fettuccine Alfredo | 1 |
| food::150::0 | What dishes have you recommended to me before? | raw | true | Rice Krispies, Jelly, Chocolate Cake | 1 |
| food::151::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Salted Caramel | 1 |
| food::152::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie, Baklava, Donuts | 1 |
| food::153::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham | 1 |
| food::154::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies | 1 |
| food::155::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee | 1 |
| food::156::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Honey Glazed Ham, Prosciutto and Melon, Sea Salt Chocolate | 1 |
| food::157::0 | What dishes have you recommended to me before? | raw | false | Ramen, Mushroom Risotto | 1 |
| food::158::0 | What dishes have you recommended to me before? | raw | false | Panna Cotta, Vanilla Milkshake | 1 |
| food::159::0 | What dishes have you recommended to me before? | raw | false | Spicy Szechuan Tofu, Cajun Shrimp | 1 |
| food::160::0 | What dishes have you recommended to me before? | raw | false | Dashi Broth, Beef Stew | 1 |
| food::161::0 | What dishes have you recommended to me before? | raw | false | Jalapeno Poppers, Salsa | 1 |
| food::162::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake | 1 |
| food::163::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi | 1 |
| food::164::0 | What dishes have you recommended to me before? | raw | false | Donuts, Honey | 1 |
| food::165::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate | 1 |
| food::166::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Chocolate Dipped Bacon | 1 |
| food::167::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Honey Glazed Ham | 1 |
| food::168::0 | What dishes have you recommended to me before? | raw | false | Spicy Tacos | 1 |
| food::169::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham, Sea Salt Chocolate | 1 |
| food::170::0 | What dishes have you recommended to me before? | raw | false | Brownies, Banana Bread | 1 |
| food::171::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::172::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Butter Toffee | 1 |
| food::173::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto, Grilled Portobello Mushrooms, Dashi Broth | 1 |
| food::174::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies | 1 |
| food::175::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::176::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::177::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Butter Toffee | 1 |
| food::178::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Maple Bacon | 1 |
| food::179::0 | What dishes have you recommended to me before? | raw | false | Parmesan Cheese | 1 |
| food::180::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce, Grilled Portobello Mushrooms | 1 |
| food::181::0 | What dishes have you recommended to me before? | raw | false | Banana Smoothie | 1 |
| food::182::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham | 1 |
| food::183::0 | What dishes have you recommended to me before? | raw | true | Prosciutto and Melon, Sea Salt Chocolate, Salted Caramel | 1 |
| food::184::0 | What dishes have you recommended to me before? | raw | false | Parmesan Cheese, Nori Seaweed, Grilled Portobello Mushrooms | 1 |
| food::185::0 | What dishes have you recommended to me before? | raw | false | Ramen | 1 |
| food::186::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Prosciutto and Melon, Maple Bacon | 1 |
| food::187::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::188::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce, Mushroom Risotto, Soy Sauce | 1 |
| food::189::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Salted Peanut Butter Cookies, Sea Salt Chocolate | 1 |
| food::190::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies | 1 |
| food::191::0 | What dishes have you recommended to me before? | raw | false | Ice Cream, Rice Pudding | 1 |
| food::192::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto, Soy Sauce | 1 |
| food::193::0 | What dishes have you recommended to me before? | raw | false | Dashi Broth | 1 |
| food::194::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie, Apple Pie, Maple Syrup Pancakes, Banana Bread | 1 |
| food::195::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Sea Salt Chocolate | 1 |
| food::196::0 | What dishes have you recommended to me before? | raw | false | Baklava, Custard, Rice Krispies, Maple Syrup Pancakes | 1 |
| food::197::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::198::0 | What dishes have you recommended to me before? | raw | false | Beef Stew, Dashi Broth, Parmesan Cheese | 1 |
| food::199::0 | What dishes have you recommended to me before? | raw | true | Chocolate Dipped Bacon, Salted Maple Ice Cream | 1 |
| food::200::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Honey Glazed Ham | 1 |
| food::201::0 | What dishes have you recommended to me before? | raw | false | Cajun Shrimp | 1 |
| food::202::0 | What dishes have you recommended to me before? | raw | true | Maple Syrup Pancakes, Candy | 1 |
| food::203::0 | What dishes have you recommended to me before? | raw | false | Mango Lassi, Coconut Pudding | 1 |
| food::204::0 | What dishes have you recommended to me before? | raw | false | Dashi Broth, Anchovy Pizza, Soy Sauce, Mushroom Risotto | 1 |
| food::205::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Sea Salt Chocolate, Pecan Praline | 1 |
| food::206::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee | 1 |
| food::207::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie | 1 |
| food::208::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Maple Bacon | 1 |
| food::209::0 | What dishes have you recommended to me before? | raw | false | BBQ Ribs, Teriyaki Sauce | 1 |
| food::210::0 | What dishes have you recommended to me before? | raw | false | Custard | 1 |
| food::211::0 | What dishes have you recommended to me before? | raw | false | Seafood, Chicken Stock, Beef Stew | 1 |
| food::212::0 | What dishes have you recommended to me before? | raw | false | Banana Bread | 1 |
| food::213::0 | What dishes have you recommended to me before? | raw | true | Prosciutto and Melon, Sea Salt Chocolate | 1 |
| food::214::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Maple Bacon, Salted Lassi | 1 |
| food::215::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi | 1 |
| food::216::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake, Banana Bread | 1 |
| food::217::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Honey Glazed Ham | 1 |
| food::218::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Prosciutto and Melon, Salted Lassi, Salted Caramel | 1 |
| food::219::0 | What dishes have you recommended to me before? | raw | false | Donuts, Maple Syrup Pancakes, Fruit | 1 |
| food::220::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Prosciutto and Melon | 1 |
| food::221::0 | What dishes have you recommended to me before? | raw | false | Banana Bread, Baklava, Rice Krispies | 1 |
| food::222::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Butterscotch Pudding | 1 |
| food::223::0 | What dishes have you recommended to me before? | raw | true | Salted Maple Ice Cream | 1 |
| food::224::0 | What dishes have you recommended to me before? | raw | true | Sweet Soy Sauce Dishes, Japanese Curry | 1 |
| food::225::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Maple Ice Cream | 1 |
| food::226::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Salted Butter Toffee, Salted Lassi, Sea Salt Chocolate | 1 |
| food::227::0 | What dishes have you recommended to me before? | raw | false | Custard, Jelly | 1 |
| food::228::0 | What dishes have you recommended to me before? | raw | false | Custard, Candy | 1 |
| food::229::0 | What dishes have you recommended to me before? | raw | false | Aged Cheddar, Nori Seaweed, Anchovy Pizza | 1 |
| food::230::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Salted Peanut Butter Cookies | 1 |
| food::231::0 | What dishes have you recommended to me before? | raw | false | Candy, Apple Pie, Chocolate Cake | 1 |
| food::232::0 | What dishes have you recommended to me before? | raw | false | Apple Pie, Maple Syrup Pancakes | 1 |
| food::233::0 | What dishes have you recommended to me before? | raw | true | Mushroom Risotto, Nori Seaweed, Grilled Portobello Mushrooms | 1 |
| food::234::0 | What dishes have you recommended to me before? | raw | false | Candy | 1 |
| food::235::0 | What dishes have you recommended to me before? | raw | true | Prosciutto and Melon, Chocolate Dipped Bacon | 1 |
| food::236::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Chocolate Dipped Bacon | 1 |
| food::237::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Salted Maple Ice Cream | 1 |
| food::238::0 | What dishes have you recommended to me before? | raw | false | Candy | 1 |
| food::239::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Prosciutto and Melon | 1 |
| food::240::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Grilled Portobello Mushrooms, Ramen | 1 |
| food::241::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Honey Glazed Ham | 1 |
| food::242::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes | 1 |
| food::243::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Salted Butterscotch Pudding, Salted Peanut Butter Cookies | 1 |
| food::244::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce, Chicken Stock | 1 |
| food::245::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Honey Glazed Ham, Salted Maple Ice Cream | 1 |
| food::246::0 | What dishes have you recommended to me before? | raw | true | Sweet and Sour Shrimp, Orange Chicken, Pineapple Fried Rice | 1 |
| food::247::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream | 1 |
| food::248::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee | 1 |
| food::249::0 | What dishes have you recommended to me before? | raw | false | Seafood, Beef Stew | 1 |
| food::250::0 | What dishes have you recommended to me before? | raw | false | Baklava, Banana Bread, Pecan Pie | 1 |
| food::251::0 | What dishes have you recommended to me before? | raw | false | Aged Cheddar, Seafood | 1 |
| food::252::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate | 1 |
| food::253::0 | What dishes have you recommended to me before? | raw | false | Fruit | 1 |
| food::254::0 | What dishes have you recommended to me before? | raw | false | Grilled Portobello Mushrooms | 1 |
| food::255::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies | 1 |
| food::256::0 | What dishes have you recommended to me before? | raw | false | Salty Crackers | 1 |
| food::257::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels, Salted Maple Ice Cream, Honey Glazed Ham, Salted Peanut Butter Cookies | 1 |
| food::258::0 | What dishes have you recommended to me before? | raw | false | Beef Stew, Ramen | 1 |
| food::259::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie | 1 |
| food::260::0 | What dishes have you recommended to me before? | raw | true | Salted Butterscotch Pudding, Salted Butter Toffee | 1 |
| food::261::0 | What dishes have you recommended to me before? | raw | true | Salted Butter Toffee, Sea Salt Chocolate | 1 |
| food::262::0 | What dishes have you recommended to me before? | raw | false | Ramen | 1 |
| food::263::0 | What dishes have you recommended to me before? | raw | false | Pork Adobo, BBQ Ribs | 1 |
| food::264::0 | What dishes have you recommended to me before? | raw | false | Candy, Honey | 1 |
| food::265::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies | 1 |
| food::266::0 | What dishes have you recommended to me before? | raw | false | Donuts, Chocolate Cake | 1 |
| food::267::0 | What dishes have you recommended to me before? | raw | false | Hoisin Glazed Duck, Balsamic Glazed Vegetables | 1 |
| food::268::0 | What dishes have you recommended to me before? | raw | false | Baklava, Jelly | 1 |
| food::269::0 | What dishes have you recommended to me before? | raw | false | Banana Bread, Custard | 1 |
| food::270::0 | What dishes have you recommended to me before? | raw | true | Salted Butterscotch Pudding, Sea Salt Chocolate, Salted Lassi | 1 |
| food::271::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Prosciutto and Melon | 1 |
| food::272::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Salted Peanut Butter Cookies | 1 |
| food::273::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Maple Ice Cream | 1 |
| food::274::0 | What dishes have you recommended to me before? | raw | false | Brownies, Donuts | 1 |
| food::275::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham | 1 |
| food::276::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Honey Glazed Ham, Maple Bacon | 1 |
| food::277::0 | What dishes have you recommended to me before? | raw | false | Candy | 1 |
| food::278::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Lassi, Salted Caramel | 1 |
| food::279::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon | 1 |
| food::280::0 | What dishes have you recommended to me before? | raw | false | Baklava | 1 |
| food::281::0 | What dishes have you recommended to me before? | raw | false | Hoisin Pork Buns, Glazed Salmon | 1 |
| food::282::0 | What dishes have you recommended to me before? | raw | false | Rice Pudding, Custard Tart | 1 |
| food::283::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Parmesan Cheese | 1 |
| food::284::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Prosciutto and Melon, Chocolate Covered Pretzels | 1 |
| food::285::0 | What dishes have you recommended to me before? | raw | false | Beef Stew, Seafood | 1 |
| food::286::0 | What dishes have you recommended to me before? | raw | false | Anchovy Pizza | 1 |
| food::287::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::288::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Chocolate Dipped Bacon, Salted Butterscotch Pudding | 1 |
| food::289::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie, Donuts | 1 |
| food::290::0 | What dishes have you recommended to me before? | raw | false | Seafood, Beef Stew, Tomato Sauce | 1 |
| food::291::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Peanut Butter Cookies | 1 |
| food::292::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Maple Bacon | 1 |
| food::293::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Donuts | 1 |
| food::294::0 | What dishes have you recommended to me before? | raw | true | Coconut Pudding, Rice Pudding | 1 |
| food::295::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto, Aged Cheddar | 1 |
| food::296::0 | What dishes have you recommended to me before? | raw | false | Baklava | 1 |
| food::297::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Maple Ice Cream | 1 |
| food::298::0 | What dishes have you recommended to me before? | raw | false | Baklava, Donuts | 1 |
| food::299::0 | What dishes have you recommended to me before? | raw | false | Brownies, Baklava | 1 |
| food::300::0 | What dishes have you recommended to me before? | raw | false | Tamarind Candy | 1 |
| food::301::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake | 1 |
| food::302::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Peanut Butter Cookies | 1 |
| food::303::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Sea Salt Chocolate | 1 |
| food::304::0 | What dishes have you recommended to me before? | raw | false | Custard, Brownies | 1 |
| food::305::0 | What dishes have you recommended to me before? | raw | false | Seafood, Mushroom Risotto | 1 |
| food::306::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels, Salted Butter Toffee | 1 |
| food::307::0 | What dishes have you recommended to me before? | raw | false | Jelly, Banana Bread | 1 |
| food::308::0 | What dishes have you recommended to me before? | raw | false | Fruit | 1 |
| food::309::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels, Salted Caramel | 1 |
| food::310::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel | 1 |
| food::311::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies | 1 |
| food::312::0 | What dishes have you recommended to me before? | raw | false | Aged Cheddar, Dashi Broth | 1 |
| food::313::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::314::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels, Salted Peanut Butter Cookies | 1 |
| food::315::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Sea Salt Chocolate | 1 |
| food::316::0 | What dishes have you recommended to me before? | raw | true | Sea Salt Chocolate | 1 |
| food::317::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Lassi, Chocolate Covered Pretzels | 1 |
| food::318::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Salted Lassi | 1 |
| food::319::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Salted Butter Toffee, Salted Maple Ice Cream, Maple Bacon | 1 |
| food::320::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Apple Pie, Jelly, Brownies | 1 |
| food::321::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies | 1 |
| food::322::0 | What dishes have you recommended to me before? | raw | true | Grilled Portobello Mushrooms, Soy Sauce, Anchovy Pizza | 1 |
| food::323::0 | What dishes have you recommended to me before? | raw | true | Donuts, Baklava | 1 |
| food::324::0 | What dishes have you recommended to me before? | raw | false | Miso Soup, Tomato Sauce | 1 |
| food::325::0 | What dishes have you recommended to me before? | raw | true | Donuts, Banana Bread | 1 |
| food::326::0 | What dishes have you recommended to me before? | raw | false | Baklava | 1 |
| food::327::0 | What dishes have you recommended to me before? | raw | false | Korean Beef Bulgogi | 1 |
| food::328::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Jelly | 1 |
| food::329::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Banana Bread, Baklava | 1 |
| food::330::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham, Salted Peanut Butter Cookies | 1 |
| food::331::0 | What dishes have you recommended to me before? | raw | false | Custard | 1 |
| food::332::0 | What dishes have you recommended to me before? | raw | false | Cheesecake, Ice Cream, Mango Lassi | 1 |
| food::333::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate | 1 |
| food::334::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Salted Peanut Butter Cookies, Salted Butter Toffee | 1 |
| food::335::0 | What dishes have you recommended to me before? | raw | true | Parmesan Cheese, Dashi Broth | 1 |
| food::336::0 | What dishes have you recommended to me before? | raw | false | Jelly, Rice Krispies | 1 |
| food::337::0 | What dishes have you recommended to me before? | raw | false | Donuts, Candy | 1 |
| food::338::0 | What dishes have you recommended to me before? | raw | true | Chocolate Cake, Fruit | 1 |
| food::339::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Lassi | 1 |
| food::340::0 | What dishes have you recommended to me before? | raw | false | Spicy Tacos | 1 |
| food::341::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto | 1 |
| food::342::0 | What dishes have you recommended to me before? | raw | false | Sweet and Sour Shrimp | 1 |
| food::343::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee | 1 |
| food::344::0 | What dishes have you recommended to me before? | raw | true | Sea Salt Chocolate, Salted Maple Ice Cream | 1 |
| food::345::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Butter Toffee | 1 |
| food::346::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Salted Butterscotch Pudding, Prosciutto and Melon | 1 |
| food::347::0 | What dishes have you recommended to me before? | raw | true | Maple Bacon, Chocolate Dipped Bacon | 1 |
| food::348::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Salted Lassi, Salted Peanut Butter Cookies | 1 |
| food::349::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Salted Butterscotch Pudding, Honey Glazed Ham | 1 |
| food::350::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Peanut Butter Cookies | 1 |
| food::351::0 | What dishes have you recommended to me before? | raw | false | Borscht Soup | 1 |
| food::352::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::353::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Salted Caramel, Salted Butter Toffee | 1 |
| food::354::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Sea Salt Chocolate | 1 |
| food::355::0 | What dishes have you recommended to me before? | raw | false | Honey, Pecan Pie, Custard | 1 |
| food::356::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Salted Butterscotch Pudding, Prosciutto and Melon | 1 |
| food::357::0 | What dishes have you recommended to me before? | raw | false | Custard, Banana Bread | 1 |
| food::358::0 | What dishes have you recommended to me before? | raw | true | Honey Glazed Ham, Prosciutto and Melon, Salted Butter Toffee | 1 |
| food::359::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake | 1 |
| food::360::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Pecan Praline | 1 |
| food::361::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Maple Bacon | 1 |
| food::362::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock | 1 |
| food::363::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Pecan Praline, Salted Lassi | 1 |
| food::364::0 | What dishes have you recommended to me before? | raw | false | Soy Sauce Marinated Eggs, Peking Duck | 1 |
| food::365::0 | What dishes have you recommended to me before? | raw | true | Salted Peanut Butter Cookies, Honey Glazed Ham, Salted Maple Ice Cream | 1 |
| food::366::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto | 1 |
| food::367::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Chocolate Dipped Bacon | 1 |
| food::368::0 | What dishes have you recommended to me before? | raw | false | Nori Seaweed, Soy Sauce | 1 |
| food::369::0 | What dishes have you recommended to me before? | raw | true | Pecan Pie, Chocolate Cake | 1 |
| food::370::0 | What dishes have you recommended to me before? | raw | false | Jelly, Chocolate Cake, Donuts | 1 |
| food::371::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto, Aged Cheddar | 1 |
| food::372::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Sea Salt Chocolate | 1 |
| food::373::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Chocolate Dipped Bacon | 1 |
| food::374::0 | What dishes have you recommended to me before? | raw | true | Salted Butterscotch Pudding, Pecan Praline | 1 |
| food::375::0 | What dishes have you recommended to me before? | raw | false | Tandoori Chicken, Spicy Ramen | 1 |
| food::376::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies | 1 |
| food::377::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake, Banana Bread | 1 |
| food::378::0 | What dishes have you recommended to me before? | raw | false | Maple Syrup Pancakes, Rice Krispies, Baklava | 1 |
| food::379::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake, Brownies | 1 |
| food::380::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Sea Salt Chocolate | 1 |
| food::381::0 | What dishes have you recommended to me before? | raw | false | Jalapeno Poppers, Chili | 1 |
| food::382::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie | 1 |
| food::383::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Butterscotch Pudding, Salted Maple Ice Cream | 1 |
| food::384::0 | What dishes have you recommended to me before? | raw | false | Candy | 1 |
| food::385::0 | What dishes have you recommended to me before? | raw | false | Aged Cheddar, Nori Seaweed | 1 |
| food::386::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Sea Salt Chocolate, Chocolate Dipped Bacon | 1 |
| food::387::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon | 1 |
| food::388::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Salted Peanut Butter Cookies | 1 |
| food::389::0 | What dishes have you recommended to me before? | raw | true | Custard | 1 |
| food::390::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi | 1 |
| food::391::0 | What dishes have you recommended to me before? | raw | false | Salsa, Cajun Shrimp, Thai Green Curry | 1 |
| food::392::0 | What dishes have you recommended to me before? | raw | false | Mushroom Risotto | 1 |
| food::393::0 | What dishes have you recommended to me before? | raw | false | Grilled Portobello Mushrooms, Anchovy Pizza | 1 |
| food::394::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce | 1 |
| food::395::0 | What dishes have you recommended to me before? | raw | false | Donuts, Pecan Pie | 1 |
| food::396::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon, Chocolate Covered Pretzels | 1 |
| food::397::0 | What dishes have you recommended to me before? | raw | false | Candy, Apple Pie, Baklava | 1 |
| food::398::0 | What dishes have you recommended to me before? | raw | false | Sugar Vinegar Ribs | 1 |
| food::399::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce | 1 |
| food::400::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi | 1 |
| food::401::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Butterscotch Pudding | 1 |
| food::402::0 | What dishes have you recommended to me before? | raw | false | Brownies | 1 |
| food::403::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::404::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Soy Sauce | 1 |
| food::405::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Pecan Praline | 1 |
| food::406::0 | What dishes have you recommended to me before? | raw | false | Ramen, Dashi Broth | 1 |
| food::407::0 | What dishes have you recommended to me before? | raw | false | Sweet and Sour Pork, Tamarind Candy | 1 |
| food::408::0 | What dishes have you recommended to me before? | raw | true | Maple Bacon, Salted Peanut Butter Cookies, Salted Caramel | 1 |
| food::409::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Honey Glazed Ham | 1 |
| food::410::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Chocolate Covered Pretzels | 1 |
| food::411::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Salted Butter Toffee | 1 |
| food::412::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels, Pecan Praline | 1 |
| food::413::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::414::0 | What dishes have you recommended to me before? | raw | false | Custard, Chocolate Cake | 1 |
| food::415::0 | What dishes have you recommended to me before? | raw | false | Mango Lassi | 1 |
| food::416::0 | What dishes have you recommended to me before? | raw | true | Salted Lassi, Salted Peanut Butter Cookies | 1 |
| food::417::0 | What dishes have you recommended to me before? | raw | false | Chocolate Cake, Honey, Rice Krispies | 1 |
| food::418::0 | What dishes have you recommended to me before? | raw | false | Ice Cream, Rice Pudding | 1 |
| food::419::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham, Salted Lassi | 1 |
| food::420::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream, Pecan Praline, Maple Bacon | 1 |
| food::421::0 | What dishes have you recommended to me before? | raw | true | Custard, Maple Syrup Pancakes | 1 |
| food::422::0 | What dishes have you recommended to me before? | raw | false | Donuts, Brownies | 1 |
| food::423::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon | 1 |
| food::424::0 | What dishes have you recommended to me before? | raw | false | Fruit | 1 |
| food::425::0 | What dishes have you recommended to me before? | raw | false | Baklava | 1 |
| food::426::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Soy Sauce | 1 |
| food::427::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream, Salted Butterscotch Pudding | 1 |
| food::428::0 | What dishes have you recommended to me before? | raw | true | Sea Salt Chocolate | 1 |
| food::429::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce | 1 |
| food::430::0 | What dishes have you recommended to me before? | raw | true | Honey Glazed Ham | 1 |
| food::431::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon | 1 |
| food::432::0 | What dishes have you recommended to me before? | raw | false | Soy Sauce, Tomato Sauce | 1 |
| food::433::0 | What dishes have you recommended to me before? | raw | false | Tomato Sauce, Aged Cheddar, Mushroom Risotto, Beef Stew | 1 |
| food::434::0 | What dishes have you recommended to me before? | raw | true | Pecan Praline | 1 |
| food::435::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels | 1 |
| food::436::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Sea Salt Chocolate | 1 |
| food::437::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream | 1 |
| food::438::0 | What dishes have you recommended to me before? | raw | false | Custard, Honey | 1 |
| food::439::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Jelly, Custard, Maple Syrup Pancakes | 1 |
| food::440::0 | What dishes have you recommended to me before? | raw | false | Donuts, Chocolate Cake | 1 |
| food::441::0 | What dishes have you recommended to me before? | raw | false | Donuts | 1 |
| food::442::0 | What dishes have you recommended to me before? | raw | false | Glazed Salmon | 1 |
| food::443::0 | What dishes have you recommended to me before? | raw | true | Maple Syrup Pancakes, Chocolate Cake, Rice Krispies | 1 |
| food::444::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham | 1 |
| food::445::0 | What dishes have you recommended to me before? | raw | false | Anchovy Pizza, Dashi Broth | 1 |
| food::446::0 | What dishes have you recommended to me before? | raw | false | Sweet and Sour Chicken, Sweet and Sour Fish | 1 |
| food::447::0 | What dishes have you recommended to me before? | raw | false | Dashi Broth, Parmesan Cheese | 1 |
| food::448::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon | 1 |
| food::449::0 | What dishes have you recommended to me before? | raw | false | Salted Maple Ice Cream, Salted Lassi | 1 |
| food::450::0 | What dishes have you recommended to me before? | raw | false | Baklava, Honey | 1 |
| food::451::0 | What dishes have you recommended to me before? | raw | true | Salted Butter Toffee | 1 |
| food::452::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Honey Glazed Ham | 1 |
| food::453::0 | What dishes have you recommended to me before? | raw | false | BBQ Ribs, Sweet Soy Sauce Dishes | 1 |
| food::454::0 | What dishes have you recommended to me before? | raw | false | Sea Salt Chocolate, Salted Maple Ice Cream | 1 |
| food::455::0 | What dishes have you recommended to me before? | raw | false | Prosciutto and Melon, Chocolate Dipped Bacon | 1 |
| food::456::0 | What dishes have you recommended to me before? | raw | true | Baklava, Honey, Rice Krispies | 1 |
| food::457::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Donuts, Chocolate Cake | 1 |
| food::458::0 | What dishes have you recommended to me before? | raw | false | Beef Stew, Tomato Sauce | 1 |
| food::459::0 | What dishes have you recommended to me before? | raw | false | Sugar Vinegar Ribs, Sweet and Sour Shrimp, Guava Jelly | 1 |
| food::460::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Salted Peanut Butter Cookies | 1 |
| food::461::0 | What dishes have you recommended to me before? | raw | false | Seafood, Grilled Portobello Mushrooms | 1 |
| food::462::0 | What dishes have you recommended to me before? | raw | true | Maple Bacon, Chocolate Dipped Bacon | 1 |
| food::463::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Maple Bacon | 1 |
| food::464::0 | What dishes have you recommended to me before? | raw | false | Pecan Pie, Custard | 1 |
| food::465::0 | What dishes have you recommended to me before? | raw | false | Rice Krispies, Apple Pie | 1 |
| food::466::0 | What dishes have you recommended to me before? | raw | true | Tandoori Chicken, Spicy Hotpot | 1 |
| food::467::0 | What dishes have you recommended to me before? | raw | false | Chocolate Covered Pretzels, Pecan Praline, Maple Bacon | 1 |
| food::468::0 | What dishes have you recommended to me before? | raw | false | Pork Adobo, Honey Soy Stir Fry | 1 |
| food::469::0 | What dishes have you recommended to me before? | raw | false | Chocolate Dipped Bacon | 1 |
| food::470::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Chocolate Covered Pretzels, Honey Glazed Ham | 1 |
| food::471::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Maple Bacon | 1 |
| food::472::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Mushroom Risotto, Miso Soup | 1 |
| food::473::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Chocolate Dipped Bacon | 1 |
| food::474::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Salted Peanut Butter Cookies, Chocolate Covered Pretzels | 1 |
| food::475::0 | What dishes have you recommended to me before? | raw | false | Soy Sauce, Aged Cheddar | 1 |
| food::476::0 | What dishes have you recommended to me before? | raw | true | Rice Krispies, Banana Bread | 1 |
| food::477::0 | What dishes have you recommended to me before? | raw | false | Chicken Stock, Anchovy Pizza | 1 |
| food::478::0 | What dishes have you recommended to me before? | raw | false | Baklava, Pecan Pie | 1 |
| food::479::0 | What dishes have you recommended to me before? | raw | false | Guava Jelly, Mango Sweet and Sour Sauce | 1 |
| food::480::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Honey Glazed Ham | 1 |
| food::481::0 | What dishes have you recommended to me before? | raw | false | Honey Glazed Ham, Maple Bacon | 1 |
| food::482::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Lassi, Salted Butterscotch Pudding | 1 |
| food::483::0 | What dishes have you recommended to me before? | raw | false | Salted Butter Toffee, Salted Peanut Butter Cookies | 1 |
| food::484::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Sea Salt Chocolate | 1 |
| food::485::0 | What dishes have you recommended to me before? | raw | false | Candy, Donuts | 1 |
| food::486::0 | What dishes have you recommended to me before? | raw | false | Pecan Praline, Maple Bacon | 1 |
| food::487::0 | What dishes have you recommended to me before? | raw | true | Salted Peanut Butter Cookies | 1 |
| food::488::0 | What dishes have you recommended to me before? | raw | false | Salted Peanut Butter Cookies, Salted Butterscotch Pudding | 1 |
| food::489::0 | What dishes have you recommended to me before? | raw | false | Beef Stew, Tomato Sauce, Seafood | 1 |
| food::490::0 | What dishes have you recommended to me before? | raw | false | Salted Caramel, Chocolate Covered Pretzels | 1 |
| food::491::0 | What dishes have you recommended to me before? | raw | true | Salted Peanut Butter Cookies, Prosciutto and Melon | 1 |
| food::492::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding | 1 |
| food::493::0 | What dishes have you recommended to me before? | raw | true | Sea Salt Chocolate, Salted Butterscotch Pudding | 1 |
| food::494::0 | What dishes have you recommended to me before? | raw | false | Maple Bacon, Pecan Praline | 1 |
| food::495::0 | What dishes have you recommended to me before? | raw | true | Chocolate Covered Pretzels | 1 |
| food::496::0 | What dishes have you recommended to me before? | raw | false | Jelly, Rice Krispies | 1 |
| food::497::0 | What dishes have you recommended to me before? | raw | true | Pecan Praline, Chocolate Dipped Bacon | 1 |
| food::498::0 | What dishes have you recommended to me before? | raw | false | Salted Butterscotch Pudding, Chocolate Dipped Bacon, Honey Glazed Ham | 1 |
| food::499::0 | What dishes have you recommended to me before? | raw | false | Salted Lassi, Salted Maple Ice Cream, Salted Caramel, Salted Butterscotch Pudding | 1 |
| movie::0::0 | What movies have you recommended to me before? | raw | false | Return of the Jedi (1983), Jurassic Park (1993) | 1 |
| movie::1::0 | What movies have you recommended to me before? | raw | true | Arsenic and Old Lace (1944), Cinema Paradiso (1988) | 1 |
| movie::2::0 | What movies have you recommended to me before? | raw | true | True Lies (1994), Boot, Das (1981) | 1 |
| movie::3::0 | What movies have you recommended to me before? | raw | false | In the Line of Fire (1993) | 1 |
| movie::4::0 | What movies have you recommended to me before? | raw | false | Contact (1997) | 1 |
| movie::5::0 | What movies have you recommended to me before? | raw | true | True Lies (1994), Great Escape, The (1963) | 1 |
| movie::6::0 | What movies have you recommended to me before? | raw | true | Twelve Monkeys (1995), Jurassic Park (1993) | 1 |
| movie::7::0 | What movies have you recommended to me before? | raw | false | Aliens (1986), Heavenly Creatures (1994) | 1 |
| movie::8::0 | What movies have you recommended to me before? | raw | false | When Harry Met Sally... (1989), Being There (1979) | 1 |
| movie::9::0 | What movies have you recommended to me before? | raw | true | Speed (1994) | 1 |
| movie::10::0 | What movies have you recommended to me before? | raw | false | Harold and Maude (1971) | 1 |
| movie::11::0 | What movies have you recommended to me before? | raw | false | Harold and Maude (1971), Wings of Desire (1987) | 1 |
| movie::12::0 | What movies have you recommended to me before? | raw | true | English Patient, The (1996), Professional, The (1994), Chasing Amy (1997) | 1 |
| movie::13::0 | What movies have you recommended to me before? | raw | true | 2001: A Space Odyssey (1968), Hunt for Red October, The (1990) | 1 |
| movie::14::0 | What movies have you recommended to me before? | raw | true | Swingers (1996) | 1 |
| movie::15::0 | What movies have you recommended to me before? | raw | false | Primal Fear (1996), Silence of the Lambs, The (1991) | 1 |
| movie::16::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969), Perfect World, A (1993) | 1 |
| movie::17::0 | What movies have you recommended to me before? | raw | false | Sting, The (1973), Roman Holiday (1953) | 1 |
| movie::18::0 | What movies have you recommended to me before? | raw | false | When Harry Met Sally... (1989), M*A*S*H (1970), Aladdin (1992) | 1 |
| movie::19::0 | What movies have you recommended to me before? | raw | false | Star Wars (1977), True Lies (1994) | 1 |
| movie::20::0 | What movies have you recommended to me before? | raw | false | Star Trek: The Wrath of Khan (1982) | 1 |
| movie::21::0 | What movies have you recommended to me before? | raw | true | Perfect World, A (1993), Magnificent Seven, The (1954) | 1 |
| movie::22::0 | What movies have you recommended to me before? | raw | false | High Noon (1952), Butch Cassidy and the Sundance Kid (1969) | 1 |
| movie::23::0 | What movies have you recommended to me before? | raw | false | Celluloid Closet, The (1995) | 1 |
| movie::24::0 | What movies have you recommended to me before? | raw | false | Crimson Tide (1995), Sling Blade (1996), Psycho (1960) | 1 |
| movie::25::0 | What movies have you recommended to me before? | raw | false | Casablanca (1942), Sense and Sensibility (1995) | 1 |
| movie::26::0 | What movies have you recommended to me before? | raw | true | Diva (1981), Abyss, The (1989) | 1 |
| movie::27::0 | What movies have you recommended to me before? | raw | false | Henry V (1989), Apocalypse Now (1979), Apt Pupil (1998) | 1 |
| movie::28::0 | What movies have you recommended to me before? | raw | false | Die Hard (1988), Face/Off (1997) | 1 |
| movie::29::0 | What movies have you recommended to me before? | raw | false | Heat (1995), Glory (1989) | 1 |
| movie::30::0 | What movies have you recommended to me before? | raw | true | 39 Steps, The (1935), Air Force One (1997), Arsenic and Old Lace (1944), Ransom (1996) | 1 |
| movie::31::0 | What movies have you recommended to me before? | raw | true | Eat Drink Man Woman (1994), Raise the Red Lantern (1991), Ran (1985) | 1 |
| movie::32::0 | What movies have you recommended to me before? | raw | false | Being There (1979) | 1 |
| movie::33::0 | What movies have you recommended to me before? | raw | false | American in Paris, An (1951) | 1 |
| movie::34::0 | What movies have you recommended to me before? | raw | true | Being There (1979) | 1 |
| movie::35::0 | What movies have you recommended to me before? | raw | true | Crimson Tide (1995), Silence of the Lambs, The (1991) | 1 |
| movie::36::0 | What movies have you recommended to me before? | raw | false | African Queen, The (1951) | 1 |
| movie::37::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987), Terminator, The (1984) | 1 |
| movie::38::0 | What movies have you recommended to me before? | raw | true | Crying Game, The (1992), Raiders of the Lost Ark (1981) | 1 |
| movie::39::0 | What movies have you recommended to me before? | raw | true | Manchurian Candidate, The (1962), Rebecca (1940) | 1 |
| movie::40::0 | What movies have you recommended to me before? | raw | false | Rock, The (1996) | 1 |
| movie::41::0 | What movies have you recommended to me before? | raw | true | Snow White and the Seven Dwarfs (1937), Pink Floyd - The Wall (1982) | 1 |
| movie::42::0 | What movies have you recommended to me before? | raw | true | Some Like It Hot (1959) | 1 |
| movie::43::0 | What movies have you recommended to me before? | raw | true | This Is Spinal Tap (1984) | 1 |
| movie::44::0 | What movies have you recommended to me before? | raw | true | Close Shave, A (1995), Young Frankenstein (1974) | 1 |
| movie::45::0 | What movies have you recommended to me before? | raw | true | Notorious (1946), Shine (1996), Princess Bride, The (1987) | 1 |
| movie::46::0 | What movies have you recommended to me before? | raw | false | Cool Hand Luke (1967) | 1 |
| movie::47::0 | What movies have you recommended to me before? | raw | false | Graduate, The (1967), Gone with the Wind (1939), Wings of Desire (1987) | 1 |
| movie::48::0 | What movies have you recommended to me before? | raw | true | Raising Arizona (1987) | 1 |
| movie::49::0 | What movies have you recommended to me before? | raw | false | Taxi Driver (1976), Die Hard (1988) | 1 |
| movie::50::0 | What movies have you recommended to me before? | raw | true | Rear Window (1954), Speed (1994) | 1 |
| movie::51::0 | What movies have you recommended to me before? | raw | false | Swingers (1996) | 1 |
| movie::52::0 | What movies have you recommended to me before? | raw | true | Three Colors: Blue (1993), Glory (1989) | 1 |
| movie::53::0 | What movies have you recommended to me before? | raw | false | Heavenly Creatures (1994), Third Man, The (1949), Taxi Driver (1976) | 1 |
| movie::54::0 | What movies have you recommended to me before? | raw | true | Professional, The (1994), Vertigo (1958) | 1 |
| movie::55::0 | What movies have you recommended to me before? | raw | true | Princess Bride, The (1987), Titanic (1997), Good, The Bad and The Ugly, The (1966) | 1 |
| movie::56::0 | What movies have you recommended to me before? | raw | false | Full Monty, The (1997) | 1 |
| movie::57::0 | What movies have you recommended to me before? | raw | false | Gattaca (1997), Close Shave, A (1995), Face/Off (1997) | 1 |
| movie::58::0 | What movies have you recommended to me before? | raw | true | Princess Bride, The (1987), Manhattan (1979), English Patient, The (1996) | 1 |
| movie::59::0 | What movies have you recommended to me before? | raw | false | E.T. the Extra-Terrestrial (1982), Jungle2Jungle (1997) | 1 |
| movie::60::0 | What movies have you recommended to me before? | raw | true | Quiet Man, The (1952), Princess Bride, The (1987), Cinema Paradiso (1988) | 1 |
| movie::61::0 | What movies have you recommended to me before? | raw | false | Aladdin (1992) | 1 |
| movie::62::0 | What movies have you recommended to me before? | raw | false | Annie Hall (1977) | 1 |
| movie::63::0 | What movies have you recommended to me before? | raw | false | In the Line of Fire (1993), Good, The Bad and The Ugly, The (1966), Empire Strikes Back, The (1980) | 1 |
| movie::64::0 | What movies have you recommended to me before? | raw | false | Man Who Would Be King, The (1975), Akira (1988), Clear and Present Danger (1994) | 1 |
| movie::65::0 | What movies have you recommended to me before? | raw | false | Quiet Man, The (1952), Princess Bride, The (1987) | 1 |
| movie::66::0 | What movies have you recommended to me before? | raw | true | Akira (1988), Back to the Future (1985) | 1 |
| movie::67::0 | What movies have you recommended to me before? | raw | false | Shallow Grave (1994) | 1 |
| movie::68::0 | What movies have you recommended to me before? | raw | true | To Kill a Mockingbird (1962) | 1 |
| movie::69::0 | What movies have you recommended to me before? | raw | false | Titanic (1997), Good, The Bad and The Ugly, The (1966), Adventures of Robin Hood, The (1938) | 1 |
| movie::70::0 | What movies have you recommended to me before? | raw | false | Face/Off (1997), Third Man, The (1949) | 1 |
| movie::71::0 | What movies have you recommended to me before? | raw | true | Ran (1985) | 1 |
| movie::72::0 | What movies have you recommended to me before? | raw | true | North by Northwest (1959) | 1 |
| movie::73::0 | What movies have you recommended to me before? | raw | true | Pink Floyd - The Wall (1982), Nightmare Before Christmas, The (1993) | 1 |
| movie::74::0 | What movies have you recommended to me before? | raw | false | Die Hard (1988) | 1 |
| movie::75::0 | What movies have you recommended to me before? | raw | false | Secret Garden, The (1993), 20,000 Leagues Under the Sea (1954) | 1 |
| movie::76::0 | What movies have you recommended to me before? | raw | false | Graduate, The (1967) | 1 |
| movie::77::0 | What movies have you recommended to me before? | raw | true | Clear and Present Danger (1994), Godfather: Part II, The (1974), Aliens (1986) | 1 |
| movie::78::0 | What movies have you recommended to me before? | raw | false | Terminator, The (1984), Sling Blade (1996) | 1 |
| movie::79::0 | What movies have you recommended to me before? | raw | false | Parent Trap, The (1961), Flubber (1997) | 1 |
| movie::80::0 | What movies have you recommended to me before? | raw | false | Swingers (1996), Cinema Paradiso (1988), Back to the Future (1985) | 1 |
| movie::81::0 | What movies have you recommended to me before? | raw | true | Shallow Grave (1994), Heavenly Creatures (1994) | 1 |
| movie::82::0 | What movies have you recommended to me before? | raw | false | Groundhog Day (1993), Chasing Amy (1997), My Fair Lady (1964), Room with a View, A (1986) | 1 |
| movie::83::0 | What movies have you recommended to me before? | raw | false | Good, The Bad and The Ugly, The (1966), Men in Black (1997) | 1 |
| movie::84::0 | What movies have you recommended to me before? | raw | true | American in Paris, An (1951), Rebecca (1940) | 1 |
| movie::85::0 | What movies have you recommended to me before? | raw | false | 39 Steps, The (1935), Face/Off (1997), Arsenic and Old Lace (1944) | 1 |
| movie::86::0 | What movies have you recommended to me before? | raw | false | Supercop (1992), Jaws (1975), Heat (1995) | 1 |
| movie::87::0 | What movies have you recommended to me before? | raw | false | Sword in the Stone, The (1963), Love Bug, The (1969) | 1 |
| movie::88::0 | What movies have you recommended to me before? | raw | true | Escape from New York (1981), 20,000 Leagues Under the Sea (1954), Back to the Future (1985) | 1 |
| movie::89::0 | What movies have you recommended to me before? | raw | true | All About Eve (1950), Three Colors: Red (1994), Lone Star (1996) | 1 |
| movie::90::0 | What movies have you recommended to me before? | raw | true | Notorious (1946), Hunt for Red October, The (1990), Apt Pupil (1998) | 1 |
| movie::91::0 | What movies have you recommended to me before? | raw | false | Face/Off (1997) | 1 |
| movie::92::0 | What movies have you recommended to me before? | raw | false | Strictly Ballroom (1992), Annie Hall (1977), Cinema Paradiso (1988) | 1 |
| movie::93::0 | What movies have you recommended to me before? | raw | true | Abyss, The (1989), Godfather: Part II, The (1974) | 1 |
| movie::94::0 | What movies have you recommended to me before? | raw | true | Strictly Ballroom (1992), Annie Hall (1977) | 1 |
| movie::95::0 | What movies have you recommended to me before? | raw | false | Raging Bull (1980), Christmas Carol, A (1938), Boot, Das (1981) | 1 |
| movie::96::0 | What movies have you recommended to me before? | raw | true | Eat Drink Man Woman (1994), Raising Arizona (1987) | 1 |
| movie::97::0 | What movies have you recommended to me before? | raw | false | Cyrano de Bergerac (1990) | 1 |
| movie::98::0 | What movies have you recommended to me before? | raw | true | Indiana Jones and the Last Crusade (1989), Star Trek: The Wrath of Khan (1982), Men in Black (1997) | 1 |
| movie::99::0 | What movies have you recommended to me before? | raw | true | In the Line of Fire (1993), Sling Blade (1996) | 1 |
| movie::100::0 | What movies have you recommended to me before? | raw | false | Three Colors: Blue (1993) | 1 |
| movie::101::0 | What movies have you recommended to me before? | raw | false | Sneakers (1992), Alien: Resurrection (1997), Star Trek VI: The Undiscovered Country (1991) | 1 |
| movie::102::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969), Tombstone (1993) | 1 |
| movie::103::0 | What movies have you recommended to me before? | raw | true | Bridge on the River Kwai, The (1957) | 1 |
| movie::104::0 | What movies have you recommended to me before? | raw | false | Harold and Maude (1971), Cinema Paradiso (1988) | 1 |
| movie::105::0 | What movies have you recommended to me before? | raw | false | Strictly Ballroom (1992), Annie Hall (1977) | 1 |
| movie::106::0 | What movies have you recommended to me before? | raw | true | To Catch a Thief (1955) | 1 |
| movie::107::0 | What movies have you recommended to me before? | raw | true | It's a Wonderful Life (1946), Boot, Das (1981) | 1 |
| movie::108::0 | What movies have you recommended to me before? | raw | false | It's a Wonderful Life (1946), Titanic (1997) | 1 |
| movie::109::0 | What movies have you recommended to me before? | raw | true | Breakfast at Tiffany's (1961), Leaving Las Vegas (1995), Manhattan (1979) | 1 |
| movie::110::0 | What movies have you recommended to me before? | raw | false | Reservoir Dogs (1992), Hoodlum (1997), Menace II Society (1993) | 1 |
| movie::111::0 | What movies have you recommended to me before? | raw | false | All About Eve (1950) | 1 |
| movie::112::0 | What movies have you recommended to me before? | raw | false | Rock, The (1996), In the Line of Fire (1993) | 1 |
| movie::113::0 | What movies have you recommended to me before? | raw | false | Notorious (1946), Sabrina (1954) | 1 |
| movie::114::0 | What movies have you recommended to me before? | raw | false | Casablanca (1942), Seven Years in Tibet (1997) | 1 |
| movie::115::0 | What movies have you recommended to me before? | raw | true | Professional, The (1994), Graduate, The (1967), Titanic (1997) | 1 |
| movie::116::0 | What movies have you recommended to me before? | raw | true | Dances with Wolves (1990), City of Lost Children, The (1995), Star Wars (1977) | 1 |
| movie::117::0 | What movies have you recommended to me before? | raw | false | Man with a Movie Camera (1929) | 1 |
| movie::118::0 | What movies have you recommended to me before? | raw | false | Boot, Das (1981) | 1 |
| movie::119::0 | What movies have you recommended to me before? | raw | false | Air Force One (1997) | 1 |
| movie::120::0 | What movies have you recommended to me before? | raw | false | Mrs. Brown (Her Majesty, Mrs. Brown) (1997) | 1 |
| movie::121::0 | What movies have you recommended to me before? | raw | false | Dances with Wolves (1990) | 1 |
| movie::122::0 | What movies have you recommended to me before? | raw | false | Die Hard (1988) | 1 |
| movie::123::0 | What movies have you recommended to me before? | raw | false | Alice in Wonderland (1951), Pocahontas (1995) | 1 |
| movie::124::0 | What movies have you recommended to me before? | raw | false | Sabrina (1954), Harold and Maude (1971), Apartment, The (1960) | 1 |
| movie::125::0 | What movies have you recommended to me before? | raw | false | Christmas Carol, A (1938), Jean de Florette (1986), Raging Bull (1980), Manon of the Spring (Manon des sources) (1986) | 1 |
| movie::126::0 | What movies have you recommended to me before? | raw | false | Fugitive, The (1993), Abyss, The (1989) | 1 |
| movie::127::0 | What movies have you recommended to me before? | raw | false | Jerry Maguire (1996) | 1 |
| movie::128::0 | What movies have you recommended to me before? | raw | false | Graduate, The (1967), Notorious (1946) | 1 |
| movie::129::0 | What movies have you recommended to me before? | raw | false | Army of Darkness (1993), Alien (1979) | 1 |
| movie::130::0 | What movies have you recommended to me before? | raw | false | Supercop (1992), Titanic (1997) | 1 |
| movie::131::0 | What movies have you recommended to me before? | raw | true | Heavenly Creatures (1994) | 1 |
| movie::132::0 | What movies have you recommended to me before? | raw | false | Raiders of the Lost Ark (1981), Rock, The (1996) | 1 |
| movie::133::0 | What movies have you recommended to me before? | raw | true | Duck Soup (1933), Sabrina (1954), Rosencrantz and Guildenstern Are Dead (1990) | 1 |
| movie::134::0 | What movies have you recommended to me before? | raw | true | Crimson Tide (1995), Face/Off (1997) | 1 |
| movie::135::0 | What movies have you recommended to me before? | raw | true | Stand by Me (1986), This Is Spinal Tap (1984) | 1 |
| movie::136::0 | What movies have you recommended to me before? | raw | false | Forbidden Planet (1956) | 1 |
| movie::137::0 | What movies have you recommended to me before? | raw | true | Manon of the Spring (Manon des sources) (1986), Wings of Desire (1987), Shawshank Redemption, The (1994), Wizard of Oz, The (1939) | 1 |
| movie::138::0 | What movies have you recommended to me before? | raw | false | Heavenly Creatures (1994), Arsenic and Old Lace (1944) | 1 |
| movie::139::0 | What movies have you recommended to me before? | raw | false | Cinema Paradiso (1988) | 1 |
| movie::140::0 | What movies have you recommended to me before? | raw | true | Fugitive, The (1993), Seven (Se7en) (1995), Professional, The (1994) | 1 |
| movie::141::0 | What movies have you recommended to me before? | raw | false | Nikita (La Femme Nikita) (1990), Dial M for Murder (1954), Red Rock West (1992) | 1 |
| movie::142::0 | What movies have you recommended to me before? | raw | false | Wings of Desire (1987) | 1 |
| movie::143::0 | What movies have you recommended to me before? | raw | false | Shadowlands (1993), Singin' in the Rain (1952), Room with a View, A (1986) | 1 |
| movie::144::0 | What movies have you recommended to me before? | raw | false | Cinema Paradiso (1988), Quiet Man, The (1952), Harold and Maude (1971) | 1 |
| movie::145::0 | What movies have you recommended to me before? | raw | true | Clear and Present Danger (1994), Star Trek: Generations (1994) | 1 |
| movie::146::0 | What movies have you recommended to me before? | raw | false | Bound (1996) | 1 |
| movie::147::0 | What movies have you recommended to me before? | raw | true | Godfather: Part II, The (1974), Three Colors: Red (1994), Silence of the Lambs, The (1991) | 1 |
| movie::148::0 | What movies have you recommended to me before? | raw | false | To Catch a Thief (1955) | 1 |
| movie::149::0 | What movies have you recommended to me before? | raw | false | Return of the Jedi (1983) | 1 |
| movie::150::0 | What movies have you recommended to me before? | raw | false | Forrest Gump (1994), Cool Hand Luke (1967) | 1 |
| movie::151::0 | What movies have you recommended to me before? | raw | true | Eat Drink Man Woman (1994), Wrong Trousers, The (1993), Toy Story (1995) | 1 |
| movie::152::0 | What movies have you recommended to me before? | raw | false | Bound (1996) | 1 |
| movie::153::0 | What movies have you recommended to me before? | raw | true | Henry V (1989), To Kill a Mockingbird (1962) | 1 |
| movie::154::0 | What movies have you recommended to me before? | raw | false | Delicatessen (1991), His Girl Friday (1940) | 1 |
| movie::155::0 | What movies have you recommended to me before? | raw | false | Wings of the Dove, The (1997), Taxi Driver (1976) | 1 |
| movie::156::0 | What movies have you recommended to me before? | raw | false | Rebecca (1940), Philadelphia Story, The (1940) | 1 |
| movie::157::0 | What movies have you recommended to me before? | raw | false | Wyatt Earp (1994), Legends of the Fall (1994) | 1 |
| movie::158::0 | What movies have you recommended to me before? | raw | false | Terminator, The (1984) | 1 |
| movie::159::0 | What movies have you recommended to me before? | raw | false | Titanic (1997), Men in Black (1997) | 1 |
| movie::160::0 | What movies have you recommended to me before? | raw | false | Client, The (1994) | 1 |
| movie::161::0 | What movies have you recommended to me before? | raw | true | Return of the Jedi (1983), True Lies (1994) | 1 |
| movie::162::0 | What movies have you recommended to me before? | raw | true | Star Trek VI: The Undiscovered Country (1991), Abyss, The (1989), City of Lost Children, The (1995) | 1 |
| movie::163::0 | What movies have you recommended to me before? | raw | false | Hoop Dreams (1994) | 1 |
| movie::164::0 | What movies have you recommended to me before? | raw | false | Raiders of the Lost Ark (1981), Jaws (1975) | 1 |
| movie::165::0 | What movies have you recommended to me before? | raw | false | Face/Off (1997), Alien (1979) | 1 |
| movie::166::0 | What movies have you recommended to me before? | raw | false | Quiet Man, The (1952) | 1 |
| movie::167::0 | What movies have you recommended to me before? | raw | true | 12 Angry Men (1957), Godfather, The (1972) | 1 |
| movie::168::0 | What movies have you recommended to me before? | raw | false | Fly Away Home (1996), Star Trek IV: The Voyage Home (1986) | 1 |
| movie::169::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987), Willy Wonka and the Chocolate Factory (1971) | 1 |
| movie::170::0 | What movies have you recommended to me before? | raw | false | Full Metal Jacket (1987), Speed (1994), Last of the Mohicans, The (1992) | 1 |
| movie::171::0 | What movies have you recommended to me before? | raw | false | Star Wars (1977) | 1 |
| movie::172::0 | What movies have you recommended to me before? | raw | true | Deer Hunter, The (1978), Dr. Strangelove or: How I Learned to Stop Worrying and Love the Bomb (1963), Mars Attacks! (1996) | 1 |
| movie::173::0 | What movies have you recommended to me before? | raw | false | African Queen, The (1951), Mrs. Brown (Her Majesty, Mrs. Brown) (1997) | 1 |
| movie::174::0 | What movies have you recommended to me before? | raw | false | Shine (1996), Chasing Amy (1997), Four Weddings and a Funeral (1994) | 1 |
| movie::175::0 | What movies have you recommended to me before? | raw | false | Titanic (1997), Room with a View, A (1986) | 1 |
| movie::176::0 | What movies have you recommended to me before? | raw | true | Some Kind of Wonderful (1987), Sense and Sensibility (1995) | 1 |
| movie::177::0 | What movies have you recommended to me before? | raw | true | Roman Holiday (1953), Duck Soup (1933), Raising Arizona (1987) | 1 |
| movie::178::0 | What movies have you recommended to me before? | raw | false | Taxi Driver (1976) | 1 |
| movie::179::0 | What movies have you recommended to me before? | raw | true | Men in Black (1997), Return of the Jedi (1983), Evil Dead II (1987) | 1 |
| movie::180::0 | What movies have you recommended to me before? | raw | false | Arsenic and Old Lace (1944), 39 Steps, The (1935) | 1 |
| movie::181::0 | What movies have you recommended to me before? | raw | false | Swingers (1996), Monty Python and the Holy Grail (1974) | 1 |
| movie::182::0 | What movies have you recommended to me before? | raw | true | L.A. Confidential (1997) | 1 |
| movie::183::0 | What movies have you recommended to me before? | raw | false | Diva (1981) | 1 |
| movie::184::0 | What movies have you recommended to me before? | raw | false | Eat Drink Man Woman (1994) | 1 |
| movie::185::0 | What movies have you recommended to me before? | raw | true | Young Frankenstein (1974), Monty Python and the Holy Grail (1974) | 1 |
| movie::186::0 | What movies have you recommended to me before? | raw | false | Casablanca (1942), Christmas Carol, A (1938), Braveheart (1995) | 1 |
| movie::187::0 | What movies have you recommended to me before? | raw | false | Raising Arizona (1987), Swingers (1996), Bringing Up Baby (1938) | 1 |
| movie::188::0 | What movies have you recommended to me before? | raw | false | Manhattan (1979), African Queen, The (1951) | 1 |
| movie::189::0 | What movies have you recommended to me before? | raw | false | Jumanji (1995) | 1 |
| movie::190::0 | What movies have you recommended to me before? | raw | true | Wizard of Oz, The (1939), Highlander (1986), Star Trek: The Wrath of Khan (1982) | 1 |
| movie::191::0 | What movies have you recommended to me before? | raw | true | Much Ado About Nothing (1993), Leaving Las Vegas (1995) | 1 |
| movie::192::0 | What movies have you recommended to me before? | raw | false | Philadelphia Story, The (1940), Much Ado About Nothing (1993), Sabrina (1954), Chasing Amy (1997) | 1 |
| movie::193::0 | What movies have you recommended to me before? | raw | true | Notorious (1946), When Harry Met Sally... (1989), Wings of Desire (1987), Return of the Jedi (1983) | 1 |
| movie::194::0 | What movies have you recommended to me before? | raw | false | Terminator, The (1984), Apt Pupil (1998), To Catch a Thief (1955) | 1 |
| movie::195::0 | What movies have you recommended to me before? | raw | false | It's a Wonderful Life (1946) | 1 |
| movie::196::0 | What movies have you recommended to me before? | raw | false | Rosencrantz and Guildenstern Are Dead (1990), North by Northwest (1959) | 1 |
| movie::197::0 | What movies have you recommended to me before? | raw | false | My Fair Lady (1964) | 1 |
| movie::198::0 | What movies have you recommended to me before? | raw | false | Local Hero (1983), Wings of Desire (1987) | 1 |
| movie::199::0 | What movies have you recommended to me before? | raw | true | Diva (1981), My Fair Lady (1964), Titanic (1997) | 1 |
| movie::200::0 | What movies have you recommended to me before? | raw | true | Citizen Kane (1941), Cool Hand Luke (1967), Empire Strikes Back, The (1980) | 1 |
| movie::201::0 | What movies have you recommended to me before? | raw | true | Aliens (1986), Sleeper (1973) | 1 |
| movie::202::0 | What movies have you recommended to me before? | raw | true | U Turn (1997), Midnight in the Garden of Good and Evil (1997) | 1 |
| movie::203::0 | What movies have you recommended to me before? | raw | false | Man with a Movie Camera (1929) | 1 |
| movie::204::0 | What movies have you recommended to me before? | raw | false | Glory (1989) | 1 |
| movie::205::0 | What movies have you recommended to me before? | raw | false | Fantasia (1940), Jumanji (1995), Aladdin (1992) | 1 |
| movie::206::0 | What movies have you recommended to me before? | raw | true | Godfather, The (1972) | 1 |
| movie::207::0 | What movies have you recommended to me before? | raw | false | Casablanca (1942), Rebecca (1940) | 1 |
| movie::208::0 | What movies have you recommended to me before? | raw | false | Celluloid Closet, The (1995) | 1 |
| movie::209::0 | What movies have you recommended to me before? | raw | true | Fargo (1996) | 1 |
| movie::210::0 | What movies have you recommended to me before? | raw | false | 12 Angry Men (1957), Eat Drink Man Woman (1994) | 1 |
| movie::211::0 | What movies have you recommended to me before? | raw | false | Godfather: Part II, The (1974), Wings of Desire (1987), Hamlet (1996) | 1 |
| movie::212::0 | What movies have you recommended to me before? | raw | false | As Good As It Gets (1997), Three Colors: Blue (1993), To Kill a Mockingbird (1962), All About Eve (1950) | 1 |
| movie::213::0 | What movies have you recommended to me before? | raw | false | Last of the Mohicans, The (1992), Godfather, The (1972) | 1 |
| movie::214::0 | What movies have you recommended to me before? | raw | false | Aristocats, The (1970) | 1 |
| movie::215::0 | What movies have you recommended to me before? | raw | false | Reservoir Dogs (1992), Crimson Tide (1995) | 1 |
| movie::216::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969), Last Man Standing (1996), Unforgiven (1992) | 1 |
| movie::217::0 | What movies have you recommended to me before? | raw | false | Jerry Maguire (1996), Rebecca (1940) | 1 |
| movie::218::0 | What movies have you recommended to me before? | raw | true | Jerry Maguire (1996), Roman Holiday (1953), Breakfast at Tiffany's (1961) | 1 |
| movie::219::0 | What movies have you recommended to me before? | raw | false | Full Monty, The (1997), Close Shave, A (1995), As Good As It Gets (1997) | 1 |
| movie::220::0 | What movies have you recommended to me before? | raw | true | Four Weddings and a Funeral (1994), Annie Hall (1977), Cyrano de Bergerac (1990) | 1 |
| movie::221::0 | What movies have you recommended to me before? | raw | true | Fargo (1996), Graduate, The (1967) | 1 |
| movie::222::0 | What movies have you recommended to me before? | raw | true | Shawshank Redemption, The (1994), Boot, Das (1981), Graduate, The (1967) | 1 |
| movie::223::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969), Supercop (1992) | 1 |
| movie::224::0 | What movies have you recommended to me before? | raw | true | 2001: A Space Odyssey (1968) | 1 |
| movie::225::0 | What movies have you recommended to me before? | raw | false | E.T. the Extra-Terrestrial (1982) | 1 |
| movie::226::0 | What movies have you recommended to me before? | raw | false | African Queen, The (1951), Crimson Tide (1995) | 1 |
| movie::227::0 | What movies have you recommended to me before? | raw | true | Alien (1979), Die Hard (1988), Heat (1995) | 1 |
| movie::228::0 | What movies have you recommended to me before? | raw | false | Adventures of Robin Hood, The (1938) | 1 |
| movie::229::0 | What movies have you recommended to me before? | raw | true | Chasing Amy (1997), African Queen, The (1951) | 1 |
| movie::230::0 | What movies have you recommended to me before? | raw | true | Babe (1995), Close Shave, A (1995) | 1 |
| movie::231::0 | What movies have you recommended to me before? | raw | false | Full Monty, The (1997), Blues Brothers, The (1980), It Happened One Night (1934) | 1 |
| movie::232::0 | What movies have you recommended to me before? | raw | false | Devil in a Blue Dress (1995), Red Corner (1997), Jackie Brown (1997) | 1 |
| movie::233::0 | What movies have you recommended to me before? | raw | true | The Thin Blue Line (1988) | 1 |
| movie::234::0 | What movies have you recommended to me before? | raw | true | Primal Fear (1996), 2001: A Space Odyssey (1968), Aliens (1986) | 1 |
| movie::235::0 | What movies have you recommended to me before? | raw | true | Notorious (1946) | 1 |
| movie::236::0 | What movies have you recommended to me before? | raw | true | Empire Strikes Back, The (1980), Quiet Man, The (1952), Room with a View, A (1986) | 1 |
| movie::237::0 | What movies have you recommended to me before? | raw | false | Return of the Jedi (1983) | 1 |
| movie::238::0 | What movies have you recommended to me before? | raw | false | Adventures of Robin Hood, The (1938), Edge, The (1997), Ben-Hur (1959) | 1 |
| movie::239::0 | What movies have you recommended to me before? | raw | false | Sense and Sensibility (1995), Leaving Las Vegas (1995) | 1 |
| movie::240::0 | What movies have you recommended to me before? | raw | false | Raising Arizona (1987), This Is Spinal Tap (1984), Wrong Trousers, The (1993) | 1 |
| movie::241::0 | What movies have you recommended to me before? | raw | true | Shadowlands (1993), Rebecca (1940), My Fair Lady (1964) | 1 |
| movie::242::0 | What movies have you recommended to me before? | raw | false | Three Colors: Blue (1993) | 1 |
| movie::243::0 | What movies have you recommended to me before? | raw | false | Hunt for Red October, The (1990) | 1 |
| movie::244::0 | What movies have you recommended to me before? | raw | true | Akira (1988), Stand by Me (1986), Star Trek: The Wrath of Khan (1982) | 1 |
| movie::245::0 | What movies have you recommended to me before? | raw | false | Emma (1996), Rebecca (1940), Like Water For Chocolate (Como agua para chocolate) (1992), Forrest Gump (1994) | 1 |
| movie::246::0 | What movies have you recommended to me before? | raw | false | To Catch a Thief (1955), Cool Hand Luke (1967) | 1 |
| movie::247::0 | What movies have you recommended to me before? | raw | false | Grand Day Out, A (1992), Local Hero (1983) | 1 |
| movie::248::0 | What movies have you recommended to me before? | raw | false | Lawrence of Arabia (1962) | 1 |
| movie::249::0 | What movies have you recommended to me before? | raw | false | Titanic (1997) | 1 |
| movie::250::0 | What movies have you recommended to me before? | raw | true | Four Weddings and a Funeral (1994), Empire Strikes Back, The (1980) | 1 |
| movie::251::0 | What movies have you recommended to me before? | raw | false | Shadowlands (1993), My Fair Lady (1964) | 1 |
| movie::252::0 | What movies have you recommended to me before? | raw | false | Manon of the Spring (Manon des sources) (1986), Cool Hand Luke (1967) | 1 |
| movie::253::0 | What movies have you recommended to me before? | raw | true | Babe (1995), Apt Pupil (1998) | 1 |
| movie::254::0 | What movies have you recommended to me before? | raw | false | Professional, The (1994), Like Water For Chocolate (Como agua para chocolate) (1992), Strictly Ballroom (1992) | 1 |
| movie::255::0 | What movies have you recommended to me before? | raw | false | Michael Collins (1996) | 1 |
| movie::256::0 | What movies have you recommended to me before? | raw | false | Braveheart (1995) | 1 |
| movie::257::0 | What movies have you recommended to me before? | raw | false | Cold Comfort Farm (1995), It Happened One Night (1934) | 1 |
| movie::258::0 | What movies have you recommended to me before? | raw | true | Diva (1981), Silence of the Lambs, The (1991), Murder in the First (1995) | 1 |
| movie::259::0 | What movies have you recommended to me before? | raw | false | Cyrano de Bergerac (1990), Wings of the Dove, The (1997) | 1 |
| movie::260::0 | What movies have you recommended to me before? | raw | false | Strictly Ballroom (1992) | 1 |
| movie::261::0 | What movies have you recommended to me before? | raw | true | Wizard of Oz, The (1939), Casablanca (1942) | 1 |
| movie::262::0 | What movies have you recommended to me before? | raw | false | Full Monty, The (1997), Apartment, The (1960), Aladdin (1992), Cool Hand Luke (1967) | 1 |
| movie::263::0 | What movies have you recommended to me before? | raw | true | Wizard of Oz, The (1939) | 1 |
| movie::264::0 | What movies have you recommended to me before? | raw | false | March of the Penguins (2005), Celluloid Closet, The (1995) | 1 |
| movie::265::0 | What movies have you recommended to me before? | raw | false | Rebecca (1940), Sabrina (1954), Like Water For Chocolate (Como agua para chocolate) (1992) | 1 |
| movie::266::0 | What movies have you recommended to me before? | raw | false | Quiet Man, The (1952), This Is Spinal Tap (1984) | 1 |
| movie::267::0 | What movies have you recommended to me before? | raw | true | Eat Drink Man Woman (1994), One Flew Over the Cuckoo's Nest (1975), Amadeus (1984) | 1 |
| movie::268::0 | What movies have you recommended to me before? | raw | false | Face/Off (1997), Fargo (1996) | 1 |
| movie::269::0 | What movies have you recommended to me before? | raw | false | Men in Black (1997) | 1 |
| movie::270::0 | What movies have you recommended to me before? | raw | true | Wizard of Oz, The (1939), Killing Fields, The (1984) | 1 |
| movie::271::0 | What movies have you recommended to me before? | raw | true | Eat Drink Man Woman (1994), This Is Spinal Tap (1984) | 1 |
| movie::272::0 | What movies have you recommended to me before? | raw | false | Gandhi (1982), Lone Star (1996) | 1 |
| movie::273::0 | What movies have you recommended to me before? | raw | false | Annie Hall (1977), Postino, Il (1994) | 1 |
| movie::274::0 | What movies have you recommended to me before? | raw | false | Singin' in the Rain (1952), Some Kind of Wonderful (1987), Princess Bride, The (1987) | 1 |
| movie::275::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969), Last Man Standing (1996) | 1 |
| movie::276::0 | What movies have you recommended to me before? | raw | true | Bringing Up Baby (1938), Forrest Gump (1994), Butch Cassidy and the Sundance Kid (1969) | 1 |
| movie::277::0 | What movies have you recommended to me before? | raw | true | True Lies (1994), Supercop (1992) | 1 |
| movie::278::0 | What movies have you recommended to me before? | raw | false | Good Will Hunting (1997), Apocalypse Now (1979) | 1 |
| movie::279::0 | What movies have you recommended to me before? | raw | false | Leaving Las Vegas (1995), Diva (1981) | 1 |
| movie::280::0 | What movies have you recommended to me before? | raw | false | Sling Blade (1996), Titanic (1997) | 1 |
| movie::281::0 | What movies have you recommended to me before? | raw | false | Sense and Sensibility (1995), Fargo (1996) | 1 |
| movie::282::0 | What movies have you recommended to me before? | raw | false | Professional, The (1994), Primal Fear (1996) | 1 |
| movie::283::0 | What movies have you recommended to me before? | raw | false | Shallow Grave (1994), Primal Fear (1996), Red Rock West (1992) | 1 |
| movie::284::0 | What movies have you recommended to me before? | raw | false | Casablanca (1942), Braveheart (1995), Patton (1970), Independence Day (ID4) (1996) | 1 |
| movie::285::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987), Aliens (1986) | 1 |
| movie::286::0 | What movies have you recommended to me before? | raw | true | It Happened One Night (1934), Harold and Maude (1971), Philadelphia Story, The (1940) | 1 |
| movie::287::0 | What movies have you recommended to me before? | raw | false | My Fair Lady (1964), Princess Bride, The (1987) | 1 |
| movie::288::0 | What movies have you recommended to me before? | raw | true | Three Colors: Red (1994), Schindler's List (1993), One Flew Over the Cuckoo's Nest (1975), Lone Star (1996) | 1 |
| movie::289::0 | What movies have you recommended to me before? | raw | false | Jurassic Park (1993), Ben-Hur (1959), Star Trek: First Contact (1996) | 1 |
| movie::290::0 | What movies have you recommended to me before? | raw | true | Babe (1995), Local Hero (1983), Clerks (1994) | 1 |
| movie::291::0 | What movies have you recommended to me before? | raw | true | Mystery Science Theater 3000: The Movie (1996), E.T. the Extra-Terrestrial (1982), Akira (1988), Independence Day (ID4) (1996) | 1 |
| movie::292::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987), Star Trek VI: The Undiscovered Country (1991) | 1 |
| movie::293::0 | What movies have you recommended to me before? | raw | false | Manhattan (1979), Strictly Ballroom (1992), Like Water For Chocolate (Como agua para chocolate) (1992) | 1 |
| movie::294::0 | What movies have you recommended to me before? | raw | false | Duck Soup (1933), Local Hero (1983) | 1 |
| movie::295::0 | What movies have you recommended to me before? | raw | true | Annie Hall (1977), Monty Python and the Holy Grail (1974), Kolya (1996) | 1 |
| movie::296::0 | What movies have you recommended to me before? | raw | true | Leaving Las Vegas (1995), Titanic (1997), My Fair Lady (1964), Notorious (1946) | 1 |
| movie::297::0 | What movies have you recommended to me before? | raw | false | Good Will Hunting (1997) | 1 |
| movie::298::0 | What movies have you recommended to me before? | raw | false | Return of the Jedi (1983), Alien: Resurrection (1997) | 1 |
| movie::299::0 | What movies have you recommended to me before? | raw | false | Speed (1994) | 1 |
| movie::300::0 | What movies have you recommended to me before? | raw | false | Glory (1989), Perfect World, A (1993) | 1 |
| movie::301::0 | What movies have you recommended to me before? | raw | true | Notorious (1946), Graduate, The (1967) | 1 |
| movie::302::0 | What movies have you recommended to me before? | raw | false | Secrets & Lies (1996), Casablanca (1942), Braveheart (1995), Sling Blade (1996) | 1 |
| movie::303::0 | What movies have you recommended to me before? | raw | false | Shadowlands (1993) | 1 |
| movie::304::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969), Monty Python and the Holy Grail (1974) | 1 |
| movie::305::0 | What movies have you recommended to me before? | raw | true | Psycho (1960), Professional, The (1994), Third Man, The (1949) | 1 |
| movie::306::0 | What movies have you recommended to me before? | raw | false | Henry V (1989), Courage Under Fire (1996) | 1 |
| movie::307::0 | What movies have you recommended to me before? | raw | false | Ransom (1996), Taxi Driver (1976) | 1 |
| movie::308::0 | What movies have you recommended to me before? | raw | false | Amadeus (1984), Wings of Desire (1987) | 1 |
| movie::309::0 | What movies have you recommended to me before? | raw | false | Postino, Il (1994), Graduate, The (1967), Groundhog Day (1993) | 1 |
| movie::310::0 | What movies have you recommended to me before? | raw | false | Godfather: Part II, The (1974), Glory (1989), Alien (1979), Aliens (1986) | 1 |
| movie::311::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987), True Lies (1994) | 1 |
| movie::312::0 | What movies have you recommended to me before? | raw | true | Gandhi (1982), Bridge on the River Kwai, The (1957) | 1 |
| movie::313::0 | What movies have you recommended to me before? | raw | false | Six Degrees of Separation (1993), Chinatown (1974) | 1 |
| movie::314::0 | What movies have you recommended to me before? | raw | false | Notorious (1946), Jerry Maguire (1996) | 1 |
| movie::315::0 | What movies have you recommended to me before? | raw | false | Eat Drink Man Woman (1994) | 1 |
| movie::316::0 | What movies have you recommended to me before? | raw | true | Swingers (1996), Local Hero (1983), North by Northwest (1959) | 1 |
| movie::317::0 | What movies have you recommended to me before? | raw | false | African Queen, The (1951), Sabrina (1954) | 1 |
| movie::318::0 | What movies have you recommended to me before? | raw | false | One Flew Over the Cuckoo's Nest (1975), Citizen Kane (1941) | 1 |
| movie::319::0 | What movies have you recommended to me before? | raw | false | Diva (1981), Apt Pupil (1998), Apollo 13 (1995) | 1 |
| movie::320::0 | What movies have you recommended to me before? | raw | false | Matilda (1996), Home Alone (1990) | 1 |
| movie::321::0 | What movies have you recommended to me before? | raw | false | E.T. the Extra-Terrestrial (1982), Sword in the Stone, The (1963) | 1 |
| movie::322::0 | What movies have you recommended to me before? | raw | false | Rock, The (1996), Good, The Bad and The Ugly, The (1966) | 1 |
| movie::323::0 | What movies have you recommended to me before? | raw | true | Good, The Bad and The Ugly, The (1966), Terminator, The (1984), Crying Game, The (1992) | 1 |
| movie::324::0 | What movies have you recommended to me before? | raw | true | Graduate, The (1967), Psycho (1960) | 1 |
| movie::325::0 | What movies have you recommended to me before? | raw | false | M*A*S*H (1970), Princess Bride, The (1987) | 1 |
| movie::326::0 | What movies have you recommended to me before? | raw | false | Terminator, The (1984), Clear and Present Danger (1994) | 1 |
| movie::327::0 | What movies have you recommended to me before? | raw | false | Like Water For Chocolate (Como agua para chocolate) (1992), Sabrina (1954) | 1 |
| movie::328::0 | What movies have you recommended to me before? | raw | false | Roman Holiday (1953), As Good As It Gets (1997) | 1 |
| movie::329::0 | What movies have you recommended to me before? | raw | false | Cool Hand Luke (1967), To Kill a Mockingbird (1962) | 1 |
| movie::330::0 | What movies have you recommended to me before? | raw | false | Three Colors: Red (1994) | 1 |
| movie::331::0 | What movies have you recommended to me before? | raw | false | To Catch a Thief (1955) | 1 |
| movie::332::0 | What movies have you recommended to me before? | raw | false | Men in Black (1997), Army of Darkness (1993), Back to the Future (1985) | 1 |
| movie::333::0 | What movies have you recommended to me before? | raw | false | My Fair Lady (1964) | 1 |
| movie::334::0 | What movies have you recommended to me before? | raw | false | Sabrina (1954), Cinema Paradiso (1988), Wrong Trousers, The (1993) | 1 |
| movie::335::0 | What movies have you recommended to me before? | raw | false | Forrest Gump (1994) | 1 |
| movie::336::0 | What movies have you recommended to me before? | raw | true | To Catch a Thief (1955), American in Paris, An (1951) | 1 |
| movie::337::0 | What movies have you recommended to me before? | raw | true | Duck Soup (1933), His Girl Friday (1940) | 1 |
| movie::338::0 | What movies have you recommended to me before? | raw | true | Great Escape, The (1963), Return of the Jedi (1983) | 1 |
| movie::339::0 | What movies have you recommended to me before? | raw | true | Kolya (1996), Wrong Trousers, The (1993), To Catch a Thief (1955) | 1 |
| movie::340::0 | What movies have you recommended to me before? | raw | false | Jungle2Jungle (1997), Jumanji (1995) | 1 |
| movie::341::0 | What movies have you recommended to me before? | raw | false | Graduate, The (1967) | 1 |
| movie::342::0 | What movies have you recommended to me before? | raw | true | Psycho (1960), Silence of the Lambs, The (1991), Rebecca (1940) | 1 |
| movie::343::0 | What movies have you recommended to me before? | raw | false | Sabrina (1954) | 1 |
| movie::344::0 | What movies have you recommended to me before? | raw | false | To Catch a Thief (1955), This Is Spinal Tap (1984) | 1 |
| movie::345::0 | What movies have you recommended to me before? | raw | false | Rosencrantz and Guildenstern Are Dead (1990), Sting, The (1973), Swingers (1996) | 1 |
| movie::346::0 | What movies have you recommended to me before? | raw | true | Jean de Florette (1986), Babe (1995) | 1 |
| movie::347::0 | What movies have you recommended to me before? | raw | false | Butch Cassidy and the Sundance Kid (1969) | 1 |
| movie::348::0 | What movies have you recommended to me before? | raw | false | Blues Brothers, The (1980), Bringing Up Baby (1938) | 1 |
| movie::349::0 | What movies have you recommended to me before? | raw | false | Silence of the Lambs, The (1991), Heavenly Creatures (1994) | 1 |
| movie::350::0 | What movies have you recommended to me before? | raw | false | Cold Comfort Farm (1995), Full Monty, The (1997), Sting, The (1973) | 1 |
| movie::351::0 | What movies have you recommended to me before? | raw | false | Men in Black (1997), Alien (1979) | 1 |
| movie::352::0 | What movies have you recommended to me before? | raw | false | Star Trek III: The Search for Spock (1984) | 1 |
| movie::353::0 | What movies have you recommended to me before? | raw | false | Jackie Brown (1997) | 1 |
| movie::354::0 | What movies have you recommended to me before? | raw | true | Glory (1989), Hunt for Red October, The (1990) | 1 |
| movie::355::0 | What movies have you recommended to me before? | raw | false | Sling Blade (1996) | 1 |
| movie::356::0 | What movies have you recommended to me before? | raw | false | Full Monty, The (1997), Local Hero (1983) | 1 |
| movie::357::0 | What movies have you recommended to me before? | raw | false | Bridge on the River Kwai, The (1957), Three Colors: Red (1994) | 1 |
| movie::358::0 | What movies have you recommended to me before? | raw | true | Adventures of Robin Hood, The (1938), Godfather: Part II, The (1974) | 1 |
| movie::359::0 | What movies have you recommended to me before? | raw | false | Murder in the First (1995), Shallow Grave (1994), Rock, The (1996) | 1 |
| movie::360::0 | What movies have you recommended to me before? | raw | false | Dances with Wolves (1990) | 1 |
| movie::361::0 | What movies have you recommended to me before? | raw | false | Taxi Driver (1976), Manon of the Spring (Manon des sources) (1986) | 1 |
| movie::362::0 | What movies have you recommended to me before? | raw | true | Magnificent Seven, The (1954), Butch Cassidy and the Sundance Kid (1969), Wyatt Earp (1994) | 1 |
| movie::363::0 | What movies have you recommended to me before? | raw | false | Boot, Das (1981), True Lies (1994), Abyss, The (1989) | 1 |
| movie::364::0 | What movies have you recommended to me before? | raw | true | Casablanca (1942), Strictly Ballroom (1992), Leaving Las Vegas (1995) | 1 |
| movie::365::0 | What movies have you recommended to me before? | raw | true | Manon of the Spring (Manon des sources) (1986), 12 Angry Men (1957), Taxi Driver (1976) | 1 |
| movie::366::0 | What movies have you recommended to me before? | raw | false | Star Wars (1977), True Romance (1993) | 1 |
| movie::367::0 | What movies have you recommended to me before? | raw | true | Get Shorty (1995), Glory (1989) | 1 |
| movie::368::0 | What movies have you recommended to me before? | raw | false | Jurassic Park (1993), Terminator, The (1984) | 1 |
| movie::369::0 | What movies have you recommended to me before? | raw | false | Forrest Gump (1994), Pink Floyd - The Wall (1982), African Queen, The (1951) | 1 |
| movie::370::0 | What movies have you recommended to me before? | raw | false | This Is Spinal Tap (1984), M*A*S*H (1970) | 1 |
| movie::371::0 | What movies have you recommended to me before? | raw | false | Wyatt Earp (1994) | 1 |
| movie::372::0 | What movies have you recommended to me before? | raw | false | Local Hero (1983), Much Ado About Nothing (1993) | 1 |
| movie::373::0 | What movies have you recommended to me before? | raw | false | Godfather: Part II, The (1974), Diva (1981) | 1 |
| movie::374::0 | What movies have you recommended to me before? | raw | false | It's a Wonderful Life (1946), Hamlet (1996), Cinema Paradiso (1988) | 1 |
| movie::375::0 | What movies have you recommended to me before? | raw | true | Cool Hand Luke (1967), Citizen Kane (1941), Jean de Florette (1986) | 1 |
| movie::376::0 | What movies have you recommended to me before? | raw | true | Schindler's List (1993), Cool Hand Luke (1967) | 1 |
| movie::377::0 | What movies have you recommended to me before? | raw | true | Chasing Amy (1997), Psycho (1960), Gone with the Wind (1939) | 1 |
| movie::378::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987), Braveheart (1995) | 1 |
| movie::379::0 | What movies have you recommended to me before? | raw | false | Alien (1979), Godfather: Part II, The (1974) | 1 |
| movie::380::0 | What movies have you recommended to me before? | raw | false | Fargo (1996) | 1 |
| movie::381::0 | What movies have you recommended to me before? | raw | false | As Good As It Gets (1997) | 1 |
| movie::382::0 | What movies have you recommended to me before? | raw | false | High Noon (1952) | 1 |
| movie::383::0 | What movies have you recommended to me before? | raw | true | Groundhog Day (1993), American in Paris, An (1951), Annie Hall (1977) | 1 |
| movie::384::0 | What movies have you recommended to me before? | raw | true | Raise the Red Lantern (1991), Raging Bull (1980), 12 Angry Men (1957) | 1 |
| movie::385::0 | What movies have you recommended to me before? | raw | false | To Catch a Thief (1955), Grand Day Out, A (1992) | 1 |
| movie::386::0 | What movies have you recommended to me before? | raw | true | Heavenly Creatures (1994), Chinatown (1974) | 1 |
| movie::387::0 | What movies have you recommended to me before? | raw | false | Men in Black (1997), Indiana Jones and the Last Crusade (1989) | 1 |
| movie::388::0 | What movies have you recommended to me before? | raw | false | M*A*S*H (1970), His Girl Friday (1940) | 1 |
| movie::389::0 | What movies have you recommended to me before? | raw | false | American in Paris, An (1951) | 1 |
| movie::390::0 | What movies have you recommended to me before? | raw | true | Star Trek: The Wrath of Khan (1982), Lawrence of Arabia (1962) | 1 |
| movie::391::0 | What movies have you recommended to me before? | raw | false | Titanic (1997), Cool Hand Luke (1967), Wizard of Oz, The (1939) | 1 |
| movie::392::0 | What movies have you recommended to me before? | raw | false | 20,000 Leagues Under the Sea (1954), Arrival, The (1996) | 1 |
| movie::393::0 | What movies have you recommended to me before? | raw | false | Fly Away Home (1996), Heavy Metal (1981) | 1 |
| movie::394::0 | What movies have you recommended to me before? | raw | true | Pulp Fiction (1994), It's a Wonderful Life (1946), Secrets & Lies (1996) | 1 |
| movie::395::0 | What movies have you recommended to me before? | raw | false | Leaving Las Vegas (1995), American in Paris, An (1951) | 1 |
| movie::396::0 | What movies have you recommended to me before? | raw | false | Adventures of Robin Hood, The (1938), Diva (1981) | 1 |
| movie::397::0 | What movies have you recommended to me before? | raw | false | Lawrence of Arabia (1962), Edge, The (1997) | 1 |
| movie::398::0 | What movies have you recommended to me before? | raw | false | Princess Bride, The (1987) | 1 |
| movie::399::0 | What movies have you recommended to me before? | raw | true | Star Wars (1977), Jerry Maguire (1996), Gone with the Wind (1939), Quiet Man, The (1952) | 1 |
| movie::400::0 | What movies have you recommended to me before? | raw | false | Star Wars (1977), Sabrina (1954), Like Water For Chocolate (Como agua para chocolate) (1992) | 1 |
| movie::401::0 | What movies have you recommended to me before? | raw | false | Annie Hall (1977), Kolya (1996) | 1 |
| movie::402::0 | What movies have you recommended to me before? | raw | false | Sense and Sensibility (1995), Room with a View, A (1986), Sabrina (1954) | 1 |
| movie::403::0 | What movies have you recommended to me before? | raw | false | Annie Hall (1977), Wings of Desire (1987), Babe (1995) | 1 |
| movie::404::0 | What movies have you recommended to me before? | raw | true | 12 Angry Men (1957), Manon of the Spring (Manon des sources) (1986) | 1 |
| movie::405::0 | What movies have you recommended to me before? | raw | true | To Catch a Thief (1955), Wings of Desire (1987) | 1 |
| movie::406::0 | What movies have you recommended to me before? | raw | false | Hercules (1997) | 1 |
| movie::407::0 | What movies have you recommended to me before? | raw | true | Breakfast at Tiffany's (1961), Cinema Paradiso (1988) | 1 |
| movie::408::0 | What movies have you recommended to me before? | raw | false | It's a Wonderful Life (1946) | 1 |
| movie::409::0 | What movies have you recommended to me before? | raw | false | Chasing Amy (1997), Empire Strikes Back, The (1980) | 1 |
| movie::410::0 | What movies have you recommended to me before? | raw | true | Crimson Tide (1995), Rock, The (1996) | 1 |
| movie::411::0 | What movies have you recommended to me before? | raw | true | Fargo (1996) | 1 |
| movie::412::0 | What movies have you recommended to me before? | raw | true | Nightmare Before Christmas, The (1993), Dumbo (1941) | 1 |
| movie::413::0 | What movies have you recommended to me before? | raw | false | Room with a View, A (1986), Leaving Las Vegas (1995) | 1 |
| movie::414::0 | What movies have you recommended to me before? | raw | false | Looking for Richard (1996), Koyaanisqatsi (1983) | 1 |
| movie::415::0 | What movies have you recommended to me before? | raw | false | Bridge on the River Kwai, The (1957) | 1 |
| movie::416::0 | What movies have you recommended to me before? | raw | false | Philadelphia Story, The (1940), Bound (1996), Much Ado About Nothing (1993) | 1 |
| movie::417::0 | What movies have you recommended to me before? | raw | false | Henry V (1989) | 1 |
| movie::418::0 | What movies have you recommended to me before? | raw | false | Babe (1995), Three Colors: Red (1994), Braveheart (1995) | 1 |
| movie::419::0 | What movies have you recommended to me before? | raw | false | To Catch a Thief (1955), Butch Cassidy and the Sundance Kid (1969), Rosencrantz and Guildenstern Are Dead (1990) | 1 |
| movie::420::0 | What movies have you recommended to me before? | raw | false | Three Musketeers, The (1993) | 1 |
| movie::421::0 | What movies have you recommended to me before? | raw | false | Cyrano de Bergerac (1990), Last of the Mohicans, The (1992) | 1 |
| movie::422::0 | What movies have you recommended to me before? | raw | false | Star Wars (1977), African Queen, The (1951), Mrs. Brown (Her Majesty, Mrs. Brown) (1997) | 1 |
| movie::423::0 | What movies have you recommended to me before? | raw | true | Leaving Las Vegas (1995), Annie Hall (1977) | 1 |
| movie::424::0 | What movies have you recommended to me before? | raw | false | Wizard of Oz, The (1939), Secret of Roan Inish, The (1994) | 1 |
| movie::425::0 | What movies have you recommended to me before? | raw | false | Philadelphia Story, The (1940) | 1 |
| movie::426::0 | What movies have you recommended to me before? | raw | false | Arsenic and Old Lace (1944), Raising Arizona (1987), M*A*S*H (1970) | 1 |
| movie::427::0 | What movies have you recommended to me before? | raw | true | Sling Blade (1996), Rear Window (1954), Shallow Grave (1994) | 1 |
| movie::428::0 | What movies have you recommended to me before? | raw | false | Star Trek: The Wrath of Khan (1982), Face/Off (1997) | 1 |
| movie::429::0 | What movies have you recommended to me before? | raw | false | Star Trek III: The Search for Spock (1984), Men in Black (1997) | 1 |
| movie::430::0 | What movies have you recommended to me before? | raw | true | Rob Roy (1995), Bananas (1971), Independence Day (ID4) (1996) | 1 |
| movie::431::0 | What movies have you recommended to me before? | raw | true | As Good As It Gets (1997), Raise the Red Lantern (1991), Wizard of Oz, The (1939) | 1 |
| movie::432::0 | What movies have you recommended to me before? | raw | false | 2001: A Space Odyssey (1968), Aliens (1986), Shallow Grave (1994) | 1 |
| movie::433::0 | What movies have you recommended to me before? | raw | false | Third Man, The (1949) | 1 |
| movie::434::0 | What movies have you recommended to me before? | raw | true | Empire Strikes Back, The (1980), Star Trek: Generations (1994) | 1 |
| movie::435::0 | What movies have you recommended to me before? | raw | false | Home Alone (1990), Casper (1995), Hunchback of Notre Dame, The (1996) | 1 |
| movie::436::0 | What movies have you recommended to me before? | raw | false | Perfect World, A (1993) | 1 |
| movie::437::0 | What movies have you recommended to me before? | raw | false | Manhattan (1979), Singin' in the Rain (1952), African Queen, The (1951) | 1 |
| movie::438::0 | What movies have you recommended to me before? | raw | false | African Queen, The (1951), Some Kind of Wonderful (1987) | 1 |
| movie::439::0 | What movies have you recommended to me before? | raw | false | Pink Floyd - The Wall (1982), Crimson Tide (1995) | 1 |
| movie::440::0 | What movies have you recommended to me before? | raw | true | Close Shave, A (1995) | 1 |
| movie::441::0 | What movies have you recommended to me before? | raw | false | Manon of the Spring (Manon des sources) (1986) | 1 |
| movie::442::0 | What movies have you recommended to me before? | raw | false | Star Trek: The Motion Picture (1979), Back to the Future (1985) | 1 |
| movie::443::0 | What movies have you recommended to me before? | raw | false | Postino, Il (1994) | 1 |
| movie::444::0 | What movies have you recommended to me before? | raw | false | Three Colors: Red (1994), Braveheart (1995) | 1 |
| movie::445::0 | What movies have you recommended to me before? | raw | false | Singin' in the Rain (1952) | 1 |
| movie::446::0 | What movies have you recommended to me before? | raw | false | Alien (1979), 20,000 Leagues Under the Sea (1954) | 1 |
| movie::447::0 | What movies have you recommended to me before? | raw | true | This Is Spinal Tap (1984), Babe (1995), Butch Cassidy and the Sundance Kid (1969) | 1 |
| movie::448::0 | What movies have you recommended to me before? | raw | false | Room with a View, A (1986), Four Weddings and a Funeral (1994) | 1 |
| movie::449::0 | What movies have you recommended to me before? | raw | false | Mrs. Brown (Her Majesty, Mrs. Brown) (1997), Gone with the Wind (1939), Sense and Sensibility (1995) | 1 |
| movie::450::0 | What movies have you recommended to me before? | raw | false | Philadelphia Story, The (1940), Much Ado About Nothing (1993) | 1 |
| movie::451::0 | What movies have you recommended to me before? | raw | false | Jerry Maguire (1996), Diva (1981) | 1 |
| movie::452::0 | What movies have you recommended to me before? | raw | false | Harold and Maude (1971) | 1 |
| movie::453::0 | What movies have you recommended to me before? | raw | false | Raiders of the Lost Ark (1981) | 1 |
| movie::454::0 | What movies have you recommended to me before? | raw | true | Diva (1981), Braveheart (1995), Indiana Jones and the Last Crusade (1989) | 1 |
| movie::455::0 | What movies have you recommended to me before? | raw | false | Star Wars (1977), Hercules (1997) | 1 |
| movie::456::0 | What movies have you recommended to me before? | raw | false | Wizard of Oz, The (1939) | 1 |
| movie::457::0 | What movies have you recommended to me before? | raw | false | North by Northwest (1959), Speed (1994), Diva (1981) | 1 |
| movie::458::0 | What movies have you recommended to me before? | raw | true | Amadeus (1984) | 1 |
| movie::459::0 | What movies have you recommended to me before? | raw | false | Bound (1996), Shadowlands (1993) | 1 |
| movie::460::0 | What movies have you recommended to me before? | raw | true | Magnificent Seven, The (1954), Boot, Das (1981) | 1 |
| movie::461::0 | What movies have you recommended to me before? | raw | false | Godfather: Part II, The (1974), Christmas Carol, A (1938) | 1 |
| movie::462::0 | What movies have you recommended to me before? | raw | false | Much Ado About Nothing (1993), Apartment, The (1960), Aladdin (1992) | 1 |
| movie::463::0 | What movies have you recommended to me before? | raw | false | 39 Steps, The (1935), Nikita (La Femme Nikita) (1990), Chinatown (1974) | 1 |
| movie::464::0 | What movies have you recommended to me before? | raw | true | Fugitive, The (1993), Rock, The (1996) | 1 |
| movie::465::0 | What movies have you recommended to me before? | raw | false | English Patient, The (1996) | 1 |
| movie::466::0 | What movies have you recommended to me before? | raw | false | Pulp Fiction (1994), Christmas Carol, A (1938), Silence of the Lambs, The (1991) | 1 |
| movie::467::0 | What movies have you recommended to me before? | raw | false | Annie Hall (1977) | 1 |
| movie::468::0 | What movies have you recommended to me before? | raw | false | Empire Strikes Back, The (1980), Emma (1996) | 1 |
| movie::469::0 | What movies have you recommended to me before? | raw | false | Shadowlands (1993) | 1 |
| movie::470::0 | What movies have you recommended to me before? | raw | false | Star Trek III: The Search for Spock (1984), Blade Runner (1982), Escape from New York (1981) | 1 |
| movie::471::0 | What movies have you recommended to me before? | raw | false | His Girl Friday (1940), Some Like It Hot (1959) | 1 |
| movie::472::0 | What movies have you recommended to me before? | raw | true | Roman Holiday (1953), Some Like It Hot (1959) | 1 |
| movie::473::0 | What movies have you recommended to me before? | raw | false | Boot, Das (1981), Three Colors: Red (1994) | 1 |
| movie::474::0 | What movies have you recommended to me before? | raw | false | Room with a View, A (1986), Quiet Man, The (1952) | 1 |
| movie::475::0 | What movies have you recommended to me before? | raw | true | Notorious (1946) | 1 |
| movie::476::0 | What movies have you recommended to me before? | raw | false | Escape from New York (1981), Around the World in 80 Days (1956) | 1 |
| movie::477::0 | What movies have you recommended to me before? | raw | false | Taxi Driver (1976) | 1 |
| movie::478::0 | What movies have you recommended to me before? | raw | true | Nikita (La Femme Nikita) (1990) | 1 |
| movie::479::0 | What movies have you recommended to me before? | raw | false | 39 Steps, The (1935), Nikita (La Femme Nikita) (1990) | 1 |
| movie::480::0 | What movies have you recommended to me before? | raw | false | Indiana Jones and the Last Crusade (1989), Boot, Das (1981) | 1 |
| movie::481::0 | What movies have you recommended to me before? | raw | false | Koyaanisqatsi (1983), Looking for Richard (1996), March of the Penguins (2005) | 1 |
| movie::482::0 | What movies have you recommended to me before? | raw | false | Rebecca (1940) | 1 |
| movie::483::0 | What movies have you recommended to me before? | raw | false | Star Trek: The Wrath of Khan (1982), Adventures of Robin Hood, The (1938), 20,000 Leagues Under the Sea (1954) | 1 |
| movie::484::0 | What movies have you recommended to me before? | raw | false | This Is Spinal Tap (1984), Duck Soup (1933) | 1 |
| movie::485::0 | What movies have you recommended to me before? | raw | true | Sting, The (1973), Wrong Trousers, The (1993) | 1 |
| movie::486::0 | What movies have you recommended to me before? | raw | false | Close Shave, A (1995), Being There (1979) | 1 |
| movie::487::0 | What movies have you recommended to me before? | raw | true | Cinema Paradiso (1988), Diva (1981) | 1 |
| movie::488::0 | What movies have you recommended to me before? | raw | false | Aliens (1986), Usual Suspects, The (1995) | 1 |
| movie::489::0 | What movies have you recommended to me before? | raw | false | Babe (1995), Grand Day Out, A (1992) | 1 |
| movie::490::0 | What movies have you recommended to me before? | raw | true | Leaving Las Vegas (1995), Room with a View, A (1986), Emma (1996) | 1 |
| movie::491::0 | What movies have you recommended to me before? | raw | false | Groundhog Day (1993), Diva (1981), Philadelphia Story, The (1940) | 1 |
| movie::492::0 | What movies have you recommended to me before? | raw | false | Boot, Das (1981), Princess Bride, The (1987) | 1 |
| movie::493::0 | What movies have you recommended to me before? | raw | false | Close Shave, A (1995) | 1 |
| movie::494::0 | What movies have you recommended to me before? | raw | false | Return of the Jedi (1983) | 1 |
| movie::495::0 | What movies have you recommended to me before? | raw | true | Wings of the Dove, The (1997), Philadelphia Story, The (1940) | 1 |
| movie::496::0 | What movies have you recommended to me before? | raw | false | Dances with Wolves (1990) | 1 |
| movie::497::0 | What movies have you recommended to me before? | raw | true | To Catch a Thief (1955), Apartment, The (1960) | 1 |
| movie::498::0 | What movies have you recommended to me before? | raw | false | Close Shave, A (1995), Speed (1994), Terminator 2: Judgment Day (1991) | 1 |
| movie::499::0 | What movies have you recommended to me before? | raw | false | African Queen, The (1951), Army of Darkness (1993), Mission: Impossible (1996) | 1 |
| multi_agent::0::0 | What movies, books and dishes have you recommended to me? | raw | false | Alien (1979), Fugitive, The (1993), Apple Pie, Naked, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| multi_agent::1::0 | What movies, books and dishes have you recommended to me? | raw | false | My Fair Lady (1964), Rebecca (1940), Banana Bread, Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It, Take Care of Yourself: The Complete Illustrated Guide to Medical Self-Care | 1 |
| multi_agent::2::0 | What movies, books and dishes have you recommended to me? | raw | false | Clockwork Orange, A (1971), Star Trek: The Motion Picture (1979), Maple Syrup Pancakes, The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss | 1 |
| multi_agent::3::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), In the Line of Fire (1993), Maple Bacon, Sea Salt Chocolate, The Nitpicker's Guide for Next Generation Trekkers, Vol. 2, Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks | 1 |
| multi_agent::4::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek: The Motion Picture (1979), Prosciutto and Melon, Sea Salt Chocolate, Chicken Soup for the Pet Lover's Soul (Chicken Soup for the Soul) | 1 |
| multi_agent::5::0 | What movies, books and dishes have you recommended to me? | raw | false | Some Kind of Wonderful (1987), Prosciutto and Melon, The Doubtful Guest | 1 |
| multi_agent::6::0 | What movies, books and dishes have you recommended to me? | raw | false | Snow White and the Seven Dwarfs (1937), Casper (1995), Chocolate Cake, Angela's Ashes (MMP) : A Memoir, A Heartbreaking Work of Staggering Genius | 2 |
| multi_agent::7::0 | What movies, books and dishes have you recommended to me? | raw | false | Ben-Hur (1959), Chocolate Dipped Bacon, One L : The Turbulent True Story of a First Year at Harvard Law School, Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States | 1 |
| multi_agent::8::0 | What movies, books and dishes have you recommended to me? | raw | false | Shine (1996), Emma (1996), Miso Soup, Soy Sauce, Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| multi_agent::9::0 | What movies, books and dishes have you recommended to me? | raw | false | Quiet Man, The (1952), Leaving Las Vegas (1995), Rice Krispies, Fruit, A Civil Action, The Cases That Haunt Us | 1 |
| multi_agent::10::0 | What movies, books and dishes have you recommended to me? | raw | false | Being There (1979), Mushroom Risotto, The Meaning Of Life | 2 |
| multi_agent::11::0 | What movies, books and dishes have you recommended to me? | raw | false | Some Like It Hot (1959), Toy Story (1995), Salted Butterscotch Pudding, Salted Butter Toffee, A Civil Action, The Cases That Haunt Us | 1 |
| multi_agent::12::0 | What movies, books and dishes have you recommended to me? | raw | false | To Catch a Thief (1955), Salted Peanut Butter Cookies, Tommo & Hawk, The Hobbit | 1 |
| multi_agent::13::0 | What movies, books and dishes have you recommended to me? | raw | false | Sense and Sensibility (1995), Leaving Las Vegas (1995), Prosciutto and Melon, Salted Butter Toffee, Songs of Innocence and Songs of Experience (Dover Thrift Editions) | 1 |
| multi_agent::14::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather: Part II, The (1974), Once Upon a Time in America (1984), Chocolate Dipped Bacon, Sea Salt Chocolate, Cats and Their Women | 1 |
| multi_agent::15::0 | What movies, books and dishes have you recommended to me? | raw | false | Arsenic and Old Lace (1944), Salted Butterscotch Pudding, Mike Nelson's Movie Megacheese | 1 |
| multi_agent::16::0 | What movies, books and dishes have you recommended to me? | raw | false | Henry V (1989), Pulp Fiction (1994), Salted Peanut Butter Cookies, Salted Maple Ice Cream, Route 66 Postcards: Greetings from the Mother Road | 1 |
| multi_agent::17::0 | What movies, books and dishes have you recommended to me? | raw | false | Indiana Jones and the Last Crusade (1989), Honey Glazed Ham, Maple Bacon, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them | 1 |
| multi_agent::18::0 | What movies, books and dishes have you recommended to me? | raw | false | Citizen Kane (1941), Ran (1985), Salted Butterscotch Pudding, Chobits Vol.1 | 1 |
| multi_agent::19::0 | What movies, books and dishes have you recommended to me? | raw | false | Sense and Sensibility (1995), Maple Bacon, Honey Glazed Ham, One L : The Turbulent True Story of a First Year at Harvard Law School | 1 |
| multi_agent::20::0 | What movies, books and dishes have you recommended to me? | raw | false | Speed (1994), Terminator, The (1984), Maple Bacon, Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It | 1 |
| multi_agent::21::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek VI: The Undiscovered Country (1991), Prosciutto and Melon, Name of the Rose | 1 |
| multi_agent::22::0 | What movies, books and dishes have you recommended to me? | raw | false | Celluloid Closet, The (1995), Bowling for Columbine (2002), Jelly, Brownies, James Herriot's Favorite Dog Stories, Cats and Their Women | 1 |
| multi_agent::23::0 | What movies, books and dishes have you recommended to me? | raw | false | Delicatessen (1991), Baklava, SEVEN HABITS OF HIGHLY EFFECTIVE PEOPLE : Powerful Lessons in Personal Change, Nickel and Dimed: On (Not) Getting By in America | 1 |
| multi_agent::24::0 | What movies, books and dishes have you recommended to me? | raw | true | His Girl Friday (1940), Fruit, Even Cowgirls Get the Blues, All Through The Night : A Suspense Story | 1 |
| multi_agent::25::0 | What movies, books and dishes have you recommended to me? | raw | false | Three Colors: Blue (1993), Gandhi (1982), Maple Bacon, Pecan Praline, So You Want to Be a Wizard: The First Book in the Young Wizards Series, El Principito | 1 |
| multi_agent::26::0 | What movies, books and dishes have you recommended to me? | raw | false | Braveheart (1995), Pecan Praline, Salted Caramel, Man's Search for Meaning: An Introduction to Logotherapy, Creative Companion: How to Free Your Creative Spirit | 1 |
| multi_agent::27::0 | What movies, books and dishes have you recommended to me? | raw | false | Full Metal Jacket (1987), Magnificent Seven, The (1954), Banana Bread, Rice Krispies, Mike Nelson's Movie Megacheese | 1 |
| multi_agent::28::0 | What movies, books and dishes have you recommended to me? | raw | false | Babe (1995), Candy, Apple Pie, One L : The Turbulent True Story of a First Year at Harvard Law School, A Civil Action | 1 |
| multi_agent::29::0 | What movies, books and dishes have you recommended to me? | raw | false | Good, The Bad and The Ugly, The (1966), Sea Salt Chocolate, Chocolate Dipped Bacon, Hard Times for These Times (English Library) | 1 |
| multi_agent::30::0 | What movies, books and dishes have you recommended to me? | raw | false | Fugitive, The (1993), Honey Glazed Ham, Salted Caramel, What to Expect When You're Expecting (Revised Edition) | 1 |
| multi_agent::31::0 | What movies, books and dishes have you recommended to me? | raw | false | Chinatown (1974), Murder in the First (1995), Chocolate Dipped Bacon, Salted Peanut Butter Cookies, In a Sunburned Country | 1 |
| multi_agent::32::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), Graduate, The (1967), Pecan Praline, A Painted House, The Da Vinci Code | 1 |
| multi_agent::33::0 | What movies, books and dishes have you recommended to me? | raw | false | Snow White and the Seven Dwarfs (1937), Evita (1996), Apple Pie, Candy, Field of Thirteen, Postmortem | 1 |
| multi_agent::34::0 | What movies, books and dishes have you recommended to me? | raw | false | Clear and Present Danger (1994), Mango Sweet and Sour Sauce, Debout les morts | 1 |
| multi_agent::35::0 | What movies, books and dishes have you recommended to me? | raw | false | Notorious (1946), Die Hard (1988), Salted Butterscotch Pudding, Lonely Planet Unpacked, Anna Karenina (Penguin Classics) | 1 |
| multi_agent::36::0 | What movies, books and dishes have you recommended to me? | raw | false | Philadelphia Story, The (1940), Fruit, Life Strategies: Doing What Works, Doing What Matters | 1 |
| multi_agent::37::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), Salted Butter Toffee, Lonely Planet Unpacked | 1 |
| multi_agent::38::0 | What movies, books and dishes have you recommended to me? | raw | false | Alien (1979), Aged Cheddar, Divine Secrets of the Ya-Ya Sisterhood: A Novel | 1 |
| multi_agent::39::0 | What movies, books and dishes have you recommended to me? | raw | false | Rock, The (1996), Pecan Pie, The Curious Sofa: A Pornographic Work by Ogdred Weary | 1 |
| multi_agent::40::0 | What movies, books and dishes have you recommended to me? | raw | false | Wyatt Earp (1994), Thai Green Curry, One L : The Turbulent True Story of a First Year at Harvard Law School, Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States | 1 |
| multi_agent::41::0 | What movies, books and dishes have you recommended to me? | raw | false | American in Paris, An (1951), Chocolate Covered Pretzels, Selected Poems (Dover Thrift Editions) | 1 |
| multi_agent::42::0 | What movies, books and dishes have you recommended to me? | raw | false | Mr. Smith Goes to Washington (1939), All About Eve (1950), Beef Stew, Miso Soup, Lies My Teacher Told Me : Everything Your American History Textbook Got Wrong | 1 |
| multi_agent::43::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather, The (1972), Maple Bacon, A Streetcar Named Desire | 1 |
| multi_agent::44::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Terminator 2: Judgment Day (1991), Sea Salt Chocolate, Chobits (Chobits), Scientific Progress Goes 'Boink':  A Calvin and Hobbes Collection | 1 |
| multi_agent::45::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), Salted Peanut Butter Cookies, Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks, The Simpsons and Philosophy: The D'oh! of Homer | 1 |
| multi_agent::46::0 | What movies, books and dishes have you recommended to me? | raw | false | Manhattan (1979), Custard, Even Cowgirls Get the Blues, The Mists of Avalon | 1 |
| multi_agent::47::0 | What movies, books and dishes have you recommended to me? | raw | false | Kolya (1996), Being There (1979), Chocolate Covered Pretzels, The Universe in a Nutshell, My Family and Other Animals. | 1 |
| multi_agent::48::0 | What movies, books and dishes have you recommended to me? | raw | false | Stand by Me (1986), Salted Maple Ice Cream, The Sorcerer's Companion: A Guide to the Magical World of Harry Potter, Lakota Woman | 1 |
| multi_agent::49::0 | What movies, books and dishes have you recommended to me? | raw | false | Blues Brothers, The (1980), Sea Salt Chocolate, Honey Glazed Ham, Lies My Teacher Told Me : Everything Your American History Textbook Got Wrong, Midnight in the Garden of Good and Evil: A Savannah Story | 1 |
| multi_agent::50::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather, The (1972), Salted Caramel, Salted Butter Toffee, The Lost Boy: A Foster Child's Search for the Love of a Family | 1 |
| multi_agent::51::0 | What movies, books and dishes have you recommended to me? | raw | false | Rob Roy (1995), Henry V (1989), Pecan Praline, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them | 1 |
| multi_agent::52::0 | What movies, books and dishes have you recommended to me? | raw | false | Close Shave, A (1995), Chocolate Dipped Bacon, A Walk in the Woods: Rediscovering America on the Appalachian Trail | 1 |
| multi_agent::53::0 | What movies, books and dishes have you recommended to me? | raw | false | Swingers (1996), Being There (1979), Candy, Fat Land: How Americans Became the Fattest People in the World, Body for Life: 12 Weeks to Mental and Physical Strength | 1 |
| multi_agent::54::0 | What movies, books and dishes have you recommended to me? | raw | false | It's a Wonderful Life (1946), Gandhi (1982), Chocolate Dipped Bacon, Ginger Tree | 1 |
| multi_agent::55::0 | What movies, books and dishes have you recommended to me? | raw | false | Aliens (1986), Godfather: Part II, The (1974), Apple Pie, Ginger Tree | 1 |
| multi_agent::56::0 | What movies, books and dishes have you recommended to me? | raw | false | Quiet Man, The (1952), Rebecca (1940), Anchovy Pizza, A Walk in the Woods: Rediscovering America on the Appalachian Trail, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| multi_agent::57::0 | What movies, books and dishes have you recommended to me? | raw | false | Braveheart (1995), Jelly, Baklava, Seinlanguage, The Darwin Awards: Evolution in Action | 1 |
| multi_agent::58::0 | What movies, books and dishes have you recommended to me? | raw | false | True Romance (1993), Indiana Jones and the Last Crusade (1989), Pecan Pie, Brownies, Dr. Atkins' New Diet Revolution, Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements | 1 |
| multi_agent::59::0 | What movies, books and dishes have you recommended to me? | raw | false | Secrets & Lies (1996), It's a Wonderful Life (1946), Salted Maple Ice Cream, A Year in Provence, A Night to Remember | 1 |
| multi_agent::60::0 | What movies, books and dishes have you recommended to me? | raw | false | Annie Hall (1977), Panna Cotta, Creme Brulee, Acqua Alta | 1 |
| multi_agent::61::0 | What movies, books and dishes have you recommended to me? | raw | false | Singin' in the Rain (1952), Salted Caramel, Prosciutto and Melon, Die Gefahrten I | 1 |
| multi_agent::62::0 | What movies, books and dishes have you recommended to me? | raw | false | Fargo (1996), Manon of the Spring (Manon des sources) (1986), Chocolate Dipped Bacon, The Nitpicker's Guide for Next Generation Trekkers, Vol. 2, Cinematherapy : The Girl's Guide to Movies for Every Mood | 1 |
| multi_agent::63::0 | What movies, books and dishes have you recommended to me? | raw | false | Raiders of the Lost Ark (1981), Prosciutto and Melon, There Are No Children Here: The Story of Two Boys Growing Up in the Other America, The Woman Warrior : Memoirs of a Girlhood Among Ghosts | 1 |
| multi_agent::64::0 | What movies, books and dishes have you recommended to me? | raw | false | Rock, The (1996), Terminator, The (1984), Apple Pie, The Te of Piglet | 1 |
| multi_agent::65::0 | What movies, books and dishes have you recommended to me? | raw | false | Groundhog Day (1993), Fruit, Banana Bread, Many Lives, Many Masters, The Mothman Prophecies | 1 |
| multi_agent::66::0 | What movies, books and dishes have you recommended to me? | raw | false | Striptease (1996), U Turn (1997), Banana Smoothie, Panna Cotta, Walden and Other Writings | 1 |
| multi_agent::67::0 | What movies, books and dishes have you recommended to me? | raw | false | Swingers (1996), Chocolate Dipped Bacon, Romeo and Juliet (Bantam Classic), Rosencrantz & Guildenstern Are Dead | 1 |
| multi_agent::68::0 | What movies, books and dishes have you recommended to me? | raw | false | Amadeus (1984), Cinema Paradiso (1988), Salted Caramel, Ginger Tree | 1 |
| multi_agent::69::0 | What movies, books and dishes have you recommended to me? | raw | false | Last of the Mohicans, The (1992), Honey, Brownies, The Tao of Pooh | 1 |
| multi_agent::70::0 | What movies, books and dishes have you recommended to me? | raw | false | Apartment, The (1960), Cold Comfort Farm (1995), Salted Butterscotch Pudding, The Law | 1 |
| multi_agent::71::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), Notorious (1946), Sea Salt Chocolate, Talking to Heaven: A Medium's Message of Life After Death | 1 |
| multi_agent::72::0 | What movies, books and dishes have you recommended to me? | raw | false | Wyatt Earp (1994), Honey Glazed Ham, Ex Libris: Confessions of a Common Reader | 1 |
| multi_agent::73::0 | What movies, books and dishes have you recommended to me? | raw | true | Good, The Bad and The Ugly, The (1966), Unforgiven (1992), Mango Lassi, More Than Complete Hitchhiker's Guide | 1 |
| multi_agent::74::0 | What movies, books and dishes have you recommended to me? | raw | false | African Queen, The (1951), Salted Maple Ice Cream, Chocolate Covered Pretzels, ANGELA'S ASHES, The Color of Water: A Black Man's Tribute to His White Mother | 1 |
| multi_agent::75::0 | What movies, books and dishes have you recommended to me? | raw | false | Jungle Book, The (1994), Wizard of Oz, The (1939), Banana Bread, Candy, Brothel: Mustang Ranch and Its Women | 1 |
| multi_agent::76::0 | What movies, books and dishes have you recommended to me? | raw | false | Aliens (1986), Sea Salt Chocolate, Lust for Life, The Doubtful Guest | 1 |
| multi_agent::77::0 | What movies, books and dishes have you recommended to me? | raw | true | North by Northwest (1959), Brownies, Catch Me If You Can: The True Story of a Real Fake, EVERYTHING SHE EVER WANTED | 1 |
| multi_agent::78::0 | What movies, books and dishes have you recommended to me? | raw | false | As Good As It Gets (1997), Rice Krispies, Beowulf: A New Verse Translation, 100 Best-Loved Poems (Dover Thrift Editions) | 1 |
| multi_agent::79::0 | What movies, books and dishes have you recommended to me? | raw | false | City of Lost Children, The (1995), Army of Darkness (1993), Pecan Praline, Anna Karenina (Penguin Classics), Women Who Run with the Wolves | 1 |
| multi_agent::80::0 | What movies, books and dishes have you recommended to me? | raw | false | Arsenic and Old Lace (1944), 2001: A Space Odyssey (1968), Teriyaki Sauce, Girl, Interrupted | 1 |
| multi_agent::81::0 | What movies, books and dishes have you recommended to me? | raw | false | Annie Hall (1977), Chocolate Dipped Bacon, Notes from a Small Island, In a Sunburned Country | 1 |
| multi_agent::82::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Blues Brothers, The (1980), Jelly, The Perfect Storm : A True Story of Men Against the Sea, The Snow Leopard (Penguin Nature Classics) | 1 |
| multi_agent::83::0 | What movies, books and dishes have you recommended to me? | raw | false | Aristocats, The (1970), Maple Bacon, Salted Butterscotch Pudding, Lies My Teacher Told Me : Everything Your American History Textbook Got Wrong | 1 |
| multi_agent::84::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Salty Crackers, Ghost World | 1 |
| multi_agent::85::0 | What movies, books and dishes have you recommended to me? | raw | false | Harold and Maude (1971), Close Shave, A (1995), Salted Caramel, Death: The High Cost of Living | 1 |
| multi_agent::86::0 | What movies, books and dishes have you recommended to me? | raw | false | Grand Day Out, A (1992), Prosciutto and Melon, ANGELA'S ASHES, Girl, Interrupted | 1 |
| multi_agent::87::0 | What movies, books and dishes have you recommended to me? | raw | true | Arsenic and Old Lace (1944), Harold and Maude (1971), Salted Lassi, Prosciutto and Melon, The Sorcerer's Companion: A Guide to the Magical World of Harry Potter | 1 |
| multi_agent::88::0 | What movies, books and dishes have you recommended to me? | raw | false | Adventures of Robin Hood, The (1938), Salted Maple Ice Cream, Mindhunter : Inside the FBI's Elite Serial Crime Unit, In a Sunburned Country | 1 |
| multi_agent::89::0 | What movies, books and dishes have you recommended to me? | raw | false | When Harry Met Sally... (1989), Much Ado About Nothing (1993), Salted Maple Ice Cream, El Senor De Los Anillos: LA Comunidad Del Anillo (Lord of the Rings (Spanish)), El Senor De Los Anillos: El Retorno Del Rey (Tolkien, J. R. R. Lord of the Rings. 3.) | 1 |
| multi_agent::90::0 | What movies, books and dishes have you recommended to me? | raw | false | Last Man Standing (1996), Good, The Bad and The Ugly, The (1966), Buffalo Wings, Jalapeno Poppers, Seabiscuit, Midnight in the Garden of Good and Evil: A Savannah Story | 1 |
| multi_agent::91::0 | What movies, books and dishes have you recommended to me? | raw | false | Psycho (1960), Salted Butterscotch Pudding, Chocolate Dipped Bacon, The Silver Chair | 1 |
| multi_agent::92::0 | What movies, books and dishes have you recommended to me? | raw | false | Secrets & Lies (1996), Amadeus (1984), Chocolate Cake, Custard, The Power of Myth | 1 |
| multi_agent::93::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather: Part II, The (1974), Alien (1979), Honey Glazed Ham, Wizard of Oz Postcards in Full Color (Card Books), The Philosophy of Andy Warhol | 1 |
| multi_agent::94::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Clockwork Orange, A (1971), Anchovy Pizza, Anna Karenina (Oprah's Book Club) | 1 |
| multi_agent::95::0 | What movies, books and dishes have you recommended to me? | raw | false | Duck Soup (1933), Anchovy Pizza, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::96::0 | What movies, books and dishes have you recommended to me? | raw | true | Much Ado About Nothing (1993), Pecan Pie, In the Heart of the Sea: The Tragedy of the Whaleship Essex | 1 |
| multi_agent::97::0 | What movies, books and dishes have you recommended to me? | raw | false | In the Line of Fire (1993), Chocolate Covered Pretzels, Honey Glazed Ham, Blind Faith | 1 |
| multi_agent::98::0 | What movies, books and dishes have you recommended to me? | raw | false | Young Frankenstein (1974), Salted Peanut Butter Cookies, Chocolate Dipped Bacon, The original Hitchhiker radio scripts, Restaurant At the End of the Universe | 1 |
| multi_agent::99::0 | What movies, books and dishes have you recommended to me? | raw | false | Quiet Man, The (1952), Titanic (1997), Jelly, Apple Pie, Nine Parts of Desire: The Hidden World of Islamic Women, The Purpose-Driven Life: What on Earth Am I Here For? | 1 |
| multi_agent::100::0 | What movies, books and dishes have you recommended to me? | raw | false | Hunt for Red October, The (1990), Face/Off (1997), Jelly, Orfe | 2 |
| multi_agent::101::0 | What movies, books and dishes have you recommended to me? | raw | false | Singin' in the Rain (1952), Salted Butter Toffee, Fraud: Essays, High Tide in Tucson : Essays from Now or Never | 1 |
| multi_agent::102::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Prosciutto and Melon, Maple Bacon, The Silver Chair, The Magician's Nephew | 1 |
| multi_agent::103::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Terminator, The (1984), Chocolate Dipped Bacon, Sense and Sensibility (World's Classics), SHIPPING NEWS | 1 |
| multi_agent::104::0 | What movies, books and dishes have you recommended to me? | raw | false | Breakfast at Tiffany's (1961), Chocolate Dipped Bacon, Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It | 1 |
| multi_agent::105::0 | What movies, books and dishes have you recommended to me? | raw | false | Jean de Florette (1986), Salted Maple Ice Cream, Chocolate Covered Pretzels, 100 Selected Poems by E. E. Cummings | 1 |
| multi_agent::106::0 | What movies, books and dishes have you recommended to me? | raw | false | Breakfast at Tiffany's (1961), Leaving Las Vegas (1995), Wasabi, Tandoori Chicken, Death of A Salesman | 2 |
| multi_agent::107::0 | What movies, books and dishes have you recommended to me? | raw | false | Groundhog Day (1993), Candy, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| multi_agent::108::0 | What movies, books and dishes have you recommended to me? | raw | false | Aliens (1986), 20,000 Leagues Under the Sea (1954), Sea Salt Chocolate, Pecan Praline, The Philosophy of Andy Warhol, Lust for Life | 1 |
| multi_agent::109::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Tomato Sauce, Grilled Portobello Mushrooms, You Just Don't Understand | 1 |
| multi_agent::110::0 | What movies, books and dishes have you recommended to me? | raw | false | Ben-Hur (1959), Brownies, When Elephants Weep: The Emotional Lives of Animals | 1 |
| multi_agent::111::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek: First Contact (1996), Anchovy Pizza, James Herriot's Dog Stories | 1 |
| multi_agent::112::0 | What movies, books and dishes have you recommended to me? | raw | false | It's a Wonderful Life (1946), Jean de Florette (1986), Salted Butter Toffee, Downsize This! Random Threats from an Unarmed American | 1 |
| multi_agent::113::0 | What movies, books and dishes have you recommended to me? | raw | false | Empire Strikes Back, The (1980), Sea Salt Chocolate, A Natural History of the Senses | 1 |
| multi_agent::114::0 | What movies, books and dishes have you recommended to me? | raw | false | Crying Game, The (1992), Great Escape, The (1963), Chocolate Covered Pretzels, Salted Maple Ice Cream, Wild Mind: Living the Writer's Life | 1 |
| multi_agent::115::0 | What movies, books and dishes have you recommended to me? | raw | false | Bridge on the River Kwai, The (1957), Henry V (1989), Ramen, Tomato Sauce, Foundations Edge | 1 |
| multi_agent::116::0 | What movies, books and dishes have you recommended to me? | raw | true | Face/Off (1997), Apollo 13 (1995), Salsa, It Was on Fire When I Lay Down on It | 1 |
| multi_agent::117::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Chocolate Dipped Bacon, The 7 Habits Of Highly Effective Teens | 1 |
| multi_agent::118::0 | What movies, books and dishes have you recommended to me? | raw | false | Air Force One (1997), Candy, Merrick (Vampire Chronicles) | 1 |
| multi_agent::119::0 | What movies, books and dishes have you recommended to me? | raw | false | Ran (1985), Beef Stew, Anna Karenina (Penguin Classics), Bibliotherapy: The Girl's Guide to Books for Every Phase of Our Lives | 1 |
| multi_agent::120::0 | What movies, books and dishes have you recommended to me? | raw | false | Men in Black (1997), Salted Lassi, In the Kitchen With Rosie: Oprah's Favorite Recipes | 1 |
| multi_agent::121::0 | What movies, books and dishes have you recommended to me? | raw | false | My Fair Lady (1964), Soy Sauce, Parmesan Cheese, Hard Times for These Times (English Library) | 1 |
| multi_agent::122::0 | What movies, books and dishes have you recommended to me? | raw | false | Butch Cassidy and the Sundance Kid (1969), Salted Peanut Butter Cookies, Amusing Ourselves to Death: Public Discourse in the Age of Show Business | 1 |
| multi_agent::123::0 | What movies, books and dishes have you recommended to me? | raw | false | Chinatown (1974), Rice Krispies, Chocolate Cake, Why Cats Paint: A Theory of Feline Aesthetics, The Iron Tonic: Or, A Winter Afternoon in Lonely Valley | 1 |
| multi_agent::124::0 | What movies, books and dishes have you recommended to me? | raw | false | Back to the Future (1985), Pecan Pie, Harry Potter and the Sorcerer's Stone (Book 1), Harry Potter and the Goblet of Fire (Book 4) | 1 |
| multi_agent::125::0 | What movies, books and dishes have you recommended to me? | raw | false | Three Musketeers, The (1993), Candy, Fruit, The Fellowship of the Ring (The Lord of the Rings, Part 1), El Senor De Los Anillos: LA Comunidad Del Anillo (Lord of the Rings (Spanish)) | 1 |
| multi_agent::126::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Salted Butter Toffee, There Are No Children Here: The Story of Two Boys Growing Up in the Other America | 1 |
| multi_agent::127::0 | What movies, books and dishes have you recommended to me? | raw | false | Taxi Driver (1976), Soy Sauce, Fat Land: How Americans Became the Fattest People in the World, Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements | 1 |
| multi_agent::128::0 | What movies, books and dishes have you recommended to me? | raw | false | James and the Giant Peach (1996), Honey Garlic Chicken, Teriyaki Sauce, Brothel: Mustang Ranch and Its Women, Lakota Woman | 1 |
| multi_agent::129::0 | What movies, books and dishes have you recommended to me? | raw | false | Babe (1995), Apt Pupil (1998), Salted Butterscotch Pudding, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times, The Tao of Pooh | 1 |
| multi_agent::130::0 | What movies, books and dishes have you recommended to me? | raw | false | Aliens (1986), Shallow Grave (1994), Chocolate Dipped Bacon, Salted Caramel, Downsize This! Random Threats from an Unarmed American, The Prince | 1 |
| multi_agent::131::0 | What movies, books and dishes have you recommended to me? | raw | false | Aliens (1986), Chocolate Dipped Bacon, Last Chance to See | 1 |
| multi_agent::132::0 | What movies, books and dishes have you recommended to me? | raw | false | Alien (1979), Custard, Maple Syrup Pancakes, Talking to Heaven: A Medium's Message of Life After Death | 1 |
| multi_agent::133::0 | What movies, books and dishes have you recommended to me? | raw | false | Graduate, The (1967), Maple Syrup Pancakes, What Should I Do with My Life? | 1 |
| multi_agent::134::0 | What movies, books and dishes have you recommended to me? | raw | false | Psycho (1960), Ramen, Flow: The Psychology of Optimal Experience, You Just Don't Understand | 1 |
| multi_agent::135::0 | What movies, books and dishes have you recommended to me? | raw | false | Notorious (1946), Face/Off (1997), Custard, Harry Potter and the Sorcerer's Stone (Book 1), Harry Potter and the Sorcerer's Stone (Harry Potter (Paperback)) | 1 |
| multi_agent::136::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek VI: The Undiscovered Country (1991), Prosciutto and Melon, Amazing Gracie: A Dog's Tale | 1 |
| multi_agent::137::0 | What movies, books and dishes have you recommended to me? | raw | false | Terminator 2: Judgment Day (1991), Terminator, The (1984), Baklava, Empire Strikes Back Wars | 1 |
| multi_agent::138::0 | What movies, books and dishes have you recommended to me? | raw | false | Face/Off (1997), Men in Black (1997), Salted Peanut Butter Cookies, Death of A Salesman, Rosencrantz & Guildenstern Are Dead | 1 |
| multi_agent::139::0 | What movies, books and dishes have you recommended to me? | raw | false | Around the World in 80 Days (1956), Pecan Pie, Brownies, The Clan of the Cave Bear : a novel, Nobilta. Commissario Brunettis siebter Fall. | 1 |
| multi_agent::140::0 | What movies, books and dishes have you recommended to me? | raw | false | Being There (1979), Local Hero (1983), Salted Butterscotch Pudding, Divine Secrets of the Ya-Ya Sisterhood: A Novel | 1 |
| multi_agent::141::0 | What movies, books and dishes have you recommended to me? | raw | false | All About Eve (1950), Citizen Kane (1941), Salted Peanut Butter Cookies, The Meaning Of Life | 1 |
| multi_agent::142::0 | What movies, books and dishes have you recommended to me? | raw | false | Casablanca (1942), Salted Butterscotch Pudding, Snow Falling on Cedars | 1 |
| multi_agent::143::0 | What movies, books and dishes have you recommended to me? | raw | false | Magnificent Seven, The (1954), Aliens (1986), Pecan Pie, The Mother Tongue | 1 |
| multi_agent::144::0 | What movies, books and dishes have you recommended to me? | raw | false | Shine (1996), Jelly, Ain't I A Woman!: A Book of Women's Poetry from Around the World | 1 |
| multi_agent::145::0 | What movies, books and dishes have you recommended to me? | raw | false | Abyss, The (1989), Custard, The Moonstone (Penguin Classics) | 1 |
| multi_agent::146::0 | What movies, books and dishes have you recommended to me? | raw | false | Crimson Tide (1995), Salted Maple Ice Cream, Salted Butter Toffee, Sense and Sensibility | 1 |
| multi_agent::147::0 | What movies, books and dishes have you recommended to me? | raw | false | 12 Angry Men (1957), To Kill a Mockingbird (1962), Spicy Szechuan Tofu, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation!, Stupid White Men ...and Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::148::0 | What movies, books and dishes have you recommended to me? | raw | false | One Flew Over the Cuckoo's Nest (1975), Pecan Praline, The Scarlet Letter: A Romance (The Penguin American Library), Ginger Tree | 1 |
| multi_agent::149::0 | What movies, books and dishes have you recommended to me? | raw | false | Butch Cassidy and the Sundance Kid (1969), Lamb Shank, Mashed Potatoes with Cream, The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss, 8 Weeks to Optimum Health | 1 |
| multi_agent::150::0 | What movies, books and dishes have you recommended to me? | raw | false | Casablanca (1942), Banana Bread, The Power of Myth, Book of Tea | 1 |
| multi_agent::151::0 | What movies, books and dishes have you recommended to me? | raw | true | Chinatown (1974), Salted Peanut Butter Cookies, Prosciutto and Melon, El Principito | 1 |
| multi_agent::152::0 | What movies, books and dishes have you recommended to me? | raw | false | Air Force One (1997), Alien (1979), Jelly, Brownies, A Royal Duty | 1 |
| multi_agent::153::0 | What movies, books and dishes have you recommended to me? | raw | false | Forrest Gump (1994), Sabrina (1954), Maple Bacon, Salted Peanut Butter Cookies, The Dark Side of the Light Chasers: Reclaiming Your Power, Creativity, Brilliance, and Dreams, Odd Girl Out: The Hidden Culture of Aggression in Girls | 1 |
| multi_agent::154::0 | What movies, books and dishes have you recommended to me? | raw | false | Strictly Ballroom (1992), Groundhog Day (1993), Pecan Praline, A Year in Provence | 1 |
| multi_agent::155::0 | What movies, books and dishes have you recommended to me? | raw | false | Philadelphia Story, The (1940), Honey Glazed Ham, Prosciutto and Melon, Creative Companion: How to Free Your Creative Spirit, You Just Don't Understand | 1 |
| multi_agent::156::0 | What movies, books and dishes have you recommended to me? | raw | false | 12 Angry Men (1957), Fargo (1996), Salted Butterscotch Pudding, An Anthropologist on Mars: Seven Paradoxical Tales, Krakatoa : The Day the World Exploded: August 27, 1883 | 1 |
| multi_agent::157::0 | What movies, books and dishes have you recommended to me? | raw | false | Glory (1989), Alien (1979), Grilled Portobello Mushrooms, Chicken Stock, Under the Tuscan Sun: At Home in Italy, A Cook's Tour | 1 |
| multi_agent::158::0 | What movies, books and dishes have you recommended to me? | raw | false | Ransom (1996), Fugitive, The (1993), Cheesecake, The Writing Life | 1 |
| multi_agent::159::0 | What movies, books and dishes have you recommended to me? | raw | false | Gattaca (1997), Tandoori Chicken, Wicca: A Guide for the Solitary Practitioner | 1 |
| multi_agent::160::0 | What movies, books and dishes have you recommended to me? | raw | false | Some Like It Hot (1959), Dashi Broth, Farewell to Manzanar: A True Story of Japanese American Experience During and  After the World War II Internment | 1 |
| multi_agent::161::0 | What movies, books and dishes have you recommended to me? | raw | false | To Kill a Mockingbird (1962), Jalapeno Poppers, Complicity | 1 |
| multi_agent::162::0 | What movies, books and dishes have you recommended to me? | raw | false | This Is Spinal Tap (1984), As Good As It Gets (1997), Jelly, Maple Syrup Pancakes, Romeo and Juliet (Bantam Classic) | 1 |
| multi_agent::163::0 | What movies, books and dishes have you recommended to me? | raw | false | Jurassic Park (1993), Salted Maple Ice Cream, The Scarlet Letter: A Romance (The Penguin American Library) | 1 |
| multi_agent::164::0 | What movies, books and dishes have you recommended to me? | raw | false | Delicatessen (1991), Quiet Man, The (1952), Maple Syrup Pancakes, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them | 1 |
| multi_agent::165::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Strictly Ballroom (1992), Honey Glazed Ham, Prosciutto and Melon, Lucky Man: A Memoir, Girl, Interrupted | 1 |
| multi_agent::166::0 | What movies, books and dishes have you recommended to me? | raw | false | Singin' in the Rain (1952), Breakfast at Tiffany's (1961), Chocolate Dipped Bacon, The Kiss | 1 |
| multi_agent::167::0 | What movies, books and dishes have you recommended to me? | raw | false | Amadeus (1984), Honey Glazed Ham, Bush at War, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::168::0 | What movies, books and dishes have you recommended to me? | raw | false | Gandhi (1982), Peri-Peri Chicken, Jalapeno Poppers, The Four Agreements: A Practical Guide to Personal Freedom, The Blue Day Book | 1 |
| multi_agent::169::0 | What movies, books and dishes have you recommended to me? | raw | true | Cool Hand Luke (1967), Much Ado About Nothing (1993), Salted Lassi, DEAD BY SUNSET : DEAD BY SUNSET | 1 |
| multi_agent::170::0 | What movies, books and dishes have you recommended to me? | raw | false | Good, The Bad and The Ugly, The (1966), Terminator, The (1984), Rice Krispies, A Midsummer Nights Dream (Bantam Classic), Death of A Salesman | 1 |
| multi_agent::171::0 | What movies, books and dishes have you recommended to me? | raw | false | Close Shave, A (1995), Salted Peanut Butter Cookies, Uncle Shelby's ABZ Book: A Primer for Adults Only | 1 |
| multi_agent::172::0 | What movies, books and dishes have you recommended to me? | raw | false | Magnificent Seven, The (1954), Salted Peanut Butter Cookies, A Walk in the Woods: Rediscovering America on the Appalachian Trail | 1 |
| multi_agent::173::0 | What movies, books and dishes have you recommended to me? | raw | false | Fargo (1996), Seafood, Anchovy Pizza, Foundations Edge | 1 |
| multi_agent::174::0 | What movies, books and dishes have you recommended to me? | raw | false | Last of the Mohicans, The (1992), Salted Lassi, The Dance of Anger: A Woman's Guide to Changing the Patterns of Intimate Relationships | 1 |
| multi_agent::175::0 | What movies, books and dishes have you recommended to me? | raw | false | Wrong Trousers, The (1993), Honey Glazed Ham, White Fang | 1 |
| multi_agent::176::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Diva (1981), Salted Maple Ice Cream, So Long and Thanks for all the Fish | 1 |
| multi_agent::177::0 | What movies, books and dishes have you recommended to me? | raw | true | Harold and Maude (1971), Prosciutto and Melon, Salted Butterscotch Pudding, Diet for a Small Planet (20th Anniversary Edition) | 1 |
| multi_agent::178::0 | What movies, books and dishes have you recommended to me? | raw | false | Adventures of Robin Hood, The (1938), Jurassic Park (1993), Salted Butter Toffee, Honey Glazed Ham, Under the Tuscan Sun: At Home in Italy | 1 |
| multi_agent::179::0 | What movies, books and dishes have you recommended to me? | raw | false | Braveheart (1995), Blues Brothers, The (1980), Beef Stew, A Streetcar Named Desire, Romeo and Juliet (Dover Thrift Editions) | 1 |
| multi_agent::180::0 | What movies, books and dishes have you recommended to me? | raw | false | Ransom (1996), Tomato Sauce, The Bad Beginning (A Series of Unfortunate Events, Book 1) | 1 |
| multi_agent::181::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Vanilla Milkshake, Notes from a Small Island, McCarthy's Bar: A Journey of Discovery In Ireland | 1 |
| multi_agent::182::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek: First Contact (1996), Salted Butter Toffee, It Was on Fire When I Lay Down on It | 1 |
| multi_agent::183::0 | What movies, books and dishes have you recommended to me? | raw | false | Serial Mom (1994), Rumble in the Bronx (1995), Chocolate Covered Pretzels, To Ride a Silver Broomstick: New Generation Witchcraft | 1 |
| multi_agent::184::0 | What movies, books and dishes have you recommended to me? | raw | false | Raising Arizona (1987), Babe (1995), Anchovy Pizza, Parmesan Cheese, Midnight in the Garden of Good and Evil: A Savannah Story, Guns, Germs, and Steel: The Fates of Human Societies | 1 |
| multi_agent::185::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Ramen, Anchovy Pizza, Dr. Atkins' New Diet Revolution, Dr. Atkins' New Diet Revolution | 1 |
| multi_agent::186::0 | What movies, books and dishes have you recommended to me? | raw | false | Toy Story (1995), Prosciutto and Melon, El Senor De Los Anillos: LA Comunidad Del Anillo (Lord of the Rings (Spanish)) | 1 |
| multi_agent::187::0 | What movies, books and dishes have you recommended to me? | raw | false | Wizard of Oz, The (1939), Prosciutto and Melon, Liebesleben, Ginger Tree | 1 |
| multi_agent::188::0 | What movies, books and dishes have you recommended to me? | raw | false | African Queen, The (1951), Seafood, Ramen, Alive : The Story of the Andes Survivors (Avon Nonfiction) | 1 |
| multi_agent::189::0 | What movies, books and dishes have you recommended to me? | raw | false | Supercop (1992), Godfather, The (1972), Salted Butter Toffee, Sea Salt Chocolate, Cosmos, My Family and Other Animals. | 1 |
| multi_agent::190::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek: First Contact (1996), Salted Maple Ice Cream, Maple Bacon, Mike Nelson's Movie Megacheese | 1 |
| multi_agent::191::0 | What movies, books and dishes have you recommended to me? | raw | false | Wings of Desire (1987), Shine (1996), Mango Lassi, Panna Cotta, Your Pregnancy: Week by Week (Your Pregnancy Series) | 1 |
| multi_agent::192::0 | What movies, books and dishes have you recommended to me? | raw | false | Cinema Paradiso (1988), Ramen, Seafood, Scientific Progress Goes 'Boink':  A Calvin and Hobbes Collection | 1 |
| multi_agent::193::0 | What movies, books and dishes have you recommended to me? | raw | false | Rear Window (1954), Dashi Broth, Miso Soup, A Sand County Almanac (Outdoor Essays & Reflections), The Man Who Listens to Horses | 1 |
| multi_agent::194::0 | What movies, books and dishes have you recommended to me? | raw | true | Braveheart (1995), Brownies, Honey, The Dark Side of the Light Chasers: Reclaiming Your Power, Creativity, Brilliance, and Dreams, Odd Girl Out: The Hidden Culture of Aggression in Girls | 1 |
| multi_agent::195::0 | What movies, books and dishes have you recommended to me? | raw | true | Secrets & Lies (1996), Prosciutto and Melon, Salted Caramel, Asta's Book | 1 |
| multi_agent::196::0 | What movies, books and dishes have you recommended to me? | raw | false | Rebecca (1940), Primal Fear (1996), Donuts, Fruit, DEAD BY SUNSET : DEAD BY SUNSET, In a Sunburned Country | 1 |
| multi_agent::197::0 | What movies, books and dishes have you recommended to me? | raw | false | Arsenic and Old Lace (1944), Chocolate Dipped Bacon, Chobits (Chobits) | 1 |
| multi_agent::198::0 | What movies, books and dishes have you recommended to me? | raw | false | Air Force One (1997), Ransom (1996), Grilled Portobello Mushrooms, The Curious Sofa: A Pornographic Work by Ogdred Weary, The Iron Tonic: Or, A Winter Afternoon in Lonely Valley | 1 |
| multi_agent::199::0 | What movies, books and dishes have you recommended to me? | raw | false | Glory (1989), Manon of the Spring (Manon des sources) (1986), Honey Glazed Ham, Salted Maple Ice Cream, The Silver Chair | 1 |
| multi_agent::200::0 | What movies, books and dishes have you recommended to me? | raw | false | Apt Pupil (1998), Maple Bacon, Sea Salt Chocolate, Fat Land: How Americans Became the Fattest People in the World | 1 |
| multi_agent::201::0 | What movies, books and dishes have you recommended to me? | raw | false | Three Colors: Red (1994), Spicy Ramen, Wasabi, This Present Darkness, Their eyes were watching God: A novel | 1 |
| multi_agent::202::0 | What movies, books and dishes have you recommended to me? | raw | false | Henry V (1989), Fruit, Banana Bread, The Street Lawyer | 1 |
| multi_agent::203::0 | What movies, books and dishes have you recommended to me? | raw | false | Philadelphia Story, The (1940), Rice Pudding, Caramel Flan, Welcome to the World Baby Girl | 1 |
| multi_agent::204::0 | What movies, books and dishes have you recommended to me? | raw | false | Harold and Maude (1971), Parmesan Cheese, Chicken Soup for the Soul (Chicken Soup for the Soul), Don't Sweat the Small Stuff and It's All Small Stuff : Simple Ways to Keep the Little Things from Taking Over Your Life (Don't Sweat the Small Stuff Series) | 1 |
| multi_agent::205::0 | What movies, books and dishes have you recommended to me? | raw | false | Ben-Hur (1959), Pecan Praline, Salted Butter Toffee, The Watcher's Guide 2 (Buffy the Vampire Slayer), The Simpsons and Philosophy: The D'oh! of Homer | 1 |
| multi_agent::206::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Salted Butterscotch Pudding, Salted Peanut Butter Cookies, The Prince, Stupid White Men ...and Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::207::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Baklava, Complicity | 1 |
| multi_agent::208::0 | What movies, books and dishes have you recommended to me? | raw | false | Dances with Wolves (1990), Chocolate Dipped Bacon, Salted Butter Toffee, Chobits (Chobits) | 1 |
| multi_agent::209::0 | What movies, books and dishes have you recommended to me? | raw | false | Local Hero (1983), It Happened One Night (1934), Honey Garlic Chicken, Coma (Signet Books) | 1 |
| multi_agent::210::0 | What movies, books and dishes have you recommended to me? | raw | false | Night of the Living Dead (1968), Jelly, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| multi_agent::211::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Anchovy Pizza, A Natural History of the Senses, The Kiss | 1 |
| multi_agent::212::0 | What movies, books and dishes have you recommended to me? | raw | false | Three Colors: Red (1994), Chocolate Cake, Fruit, Lucky Man: A Memoir, Girl, Interrupted | 1 |
| multi_agent::213::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Maple Bacon, Salted Peanut Butter Cookies, Plain and Simple : A Journey to the Amish (Ohio), This Present Darkness | 1 |
| multi_agent::214::0 | What movies, books and dishes have you recommended to me? | raw | false | Quiet Man, The (1952), Grand Day Out, A (1992), Salted Caramel, Salted Maple Ice Cream, Talking to Heaven: A Medium's Message of Life After Death, Many Lives, Many Masters | 1 |
| multi_agent::215::0 | What movies, books and dishes have you recommended to me? | raw | false | Good Will Hunting (1997), Godfather: Part II, The (1974), Honey Glazed Ham, Prince and the Pauper Walt Disney, White Fang | 1 |
| multi_agent::216::0 | What movies, books and dishes have you recommended to me? | raw | false | Unforgiven (1992), Wyatt Earp (1994), Maple Syrup Pancakes, Chicken Soup for the Teenage Soul II (Chicken Soup for the Soul Series) | 1 |
| multi_agent::217::0 | What movies, books and dishes have you recommended to me? | raw | false | Annie Hall (1977), Philadelphia Story, The (1940), Sea Salt Chocolate, Salted Maple Ice Cream, Divine Secrets of the Ya-Ya Sisterhood: A Novel | 1 |
| multi_agent::218::0 | What movies, books and dishes have you recommended to me? | raw | false | Singin' in the Rain (1952), Salted Caramel, Kitchen | 1 |
| multi_agent::219::0 | What movies, books and dishes have you recommended to me? | raw | false | Dances with Wolves (1990), Honey, Rice Krispies, A Rose For Her Grave & Other True Cases (Ann Rule's Crime Files), The Writing Life | 1 |
| multi_agent::220::0 | What movies, books and dishes have you recommended to me? | raw | false | True Lies (1994), Maple Bacon, Honey Glazed Ham, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them | 1 |
| multi_agent::221::0 | What movies, books and dishes have you recommended to me? | raw | false | Blues Brothers, The (1980), Maple Syrup Pancakes, Brownies, A Wrinkle In Time, Harry Potter and the Goblet of Fire (Book 4) | 1 |
| multi_agent::222::0 | What movies, books and dishes have you recommended to me? | raw | false | Three Musketeers, The (1993), Salted Lassi, Chocolate Covered Pretzels, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::223::0 | What movies, books and dishes have you recommended to me? | raw | false | Apollo 13 (1995), Salted Butter Toffee, The Tao of Pooh, Book of Tea | 1 |
| multi_agent::224::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather: Part II, The (1974), Hoisin Glazed Duck, Japanese Sweet Egg Tamagoyaki, Book of Virtues, The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child | 1 |
| multi_agent::225::0 | What movies, books and dishes have you recommended to me? | raw | false | Sabrina (1954), Salted Maple Ice Cream, Foundations Edge, The Greatest Show Off Earth | 1 |
| multi_agent::226::0 | What movies, books and dishes have you recommended to me? | raw | false | Blues Brothers, The (1980), Pecan Praline, The Secret Life of Bees | 1 |
| multi_agent::227::0 | What movies, books and dishes have you recommended to me? | raw | false | True Romance (1993), Jelly, Liebesleben | 1 |
| multi_agent::228::0 | What movies, books and dishes have you recommended to me? | raw | false | Four Weddings and a Funeral (1994), Shadowlands (1993), Rice Krispies, Zen in the Art of Writing, The Mother Tongue | 1 |
| multi_agent::229::0 | What movies, books and dishes have you recommended to me? | raw | false | Kolya (1996), Tomato Sauce, Walden and Other Writings, Fraud: Essays | 1 |
| multi_agent::230::0 | What movies, books and dishes have you recommended to me? | raw | false | True Romance (1993), Salted Butterscotch Pudding, No Bad Dogs : The Woodhouse Way, James Herriot's Favorite Dog Stories | 1 |
| multi_agent::231::0 | What movies, books and dishes have you recommended to me? | raw | false | Wings of Desire (1987), Notorious (1946), Chocolate Cake, Different Seasons | 1 |
| multi_agent::232::0 | What movies, books and dishes have you recommended to me? | raw | false | Diva (1981), Banana Bread, Anna Karenina (Penguin Classics), Book Lust: Recommended Reading for Every Mood, Moment, and Reason | 1 |
| multi_agent::233::0 | What movies, books and dishes have you recommended to me? | raw | false | Much Ado About Nothing (1993), Forrest Gump (1994), Grilled Portobello Mushrooms, Ramen, Brothel: Mustang Ranch and Its Women, You Just Don't Understand | 1 |
| multi_agent::234::0 | What movies, books and dishes have you recommended to me? | raw | false | Hunt for Red October, The (1990), Heavenly Creatures (1994), Custard, Chocolate Cake, Under the Tuscan Sun | 1 |
| multi_agent::235::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Raiders of the Lost Ark (1981), Honey Glazed Ham, Nine Parts of Desire: The Hidden World of Islamic Women | 1 |
| multi_agent::236::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Honey Glazed Ham, Death: The High Cost of Living, Watchmen | 1 |
| multi_agent::237::0 | What movies, books and dishes have you recommended to me? | raw | false | Magnificent Seven, The (1954), Perfect World, A (1993), Prosciutto and Melon, An Anthropologist on Mars: Seven Paradoxical Tales | 1 |
| multi_agent::238::0 | What movies, books and dishes have you recommended to me? | raw | false | Duck Soup (1933), Banana Bread, Honey, Postmortem, Asta's Book | 1 |
| multi_agent::239::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Salted Caramel, Chocolate Covered Pretzels, Savage Inequalities: Children in America's Schools | 1 |
| multi_agent::240::0 | What movies, books and dishes have you recommended to me? | raw | false | Good Will Hunting (1997), Aged Cheddar, The Cases That Haunt Us | 1 |
| multi_agent::241::0 | What movies, books and dishes have you recommended to me? | raw | false | Blues Brothers, The (1980), Salted Butter Toffee, Girl, Interrupted, Tuesdays with Morrie: An Old Man, a Young Man, and Life's Greatest Lesson | 1 |
| multi_agent::242::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek: First Contact (1996), Braveheart (1995), Baklava, Even Cowgirls Get the Blues | 1 |
| multi_agent::243::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Apollo 13 (1995), Chocolate Covered Pretzels, Fish! A Remarkable Way to Boost Morale and Improve Results | 1 |
| multi_agent::244::0 | What movies, books and dishes have you recommended to me? | raw | false | One Flew Over the Cuckoo's Nest (1975), Godfather: Part II, The (1974), Parmesan Cheese, The Silence of the Lambs, Empire Strikes Back Wars | 1 |
| multi_agent::245::0 | What movies, books and dishes have you recommended to me? | raw | false | Stargate (1994), Salted Caramel, Behind the Scenes at the Museum | 1 |
| multi_agent::246::0 | What movies, books and dishes have you recommended to me? | raw | false | Shawshank Redemption, The (1994), Sweet and Sour Fish, Tamarind Candy, A Civil Action, The Cases That Haunt Us | 1 |
| multi_agent::247::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather: Part II, The (1974), Salted Butterscotch Pudding, Salted Maple Ice Cream, The Grey King (The Dark is Rising Sequence) | 1 |
| multi_agent::248::0 | What movies, books and dishes have you recommended to me? | raw | false | Man Who Would Be King, The (1975), Chocolate Covered Pretzels, Honey Glazed Ham, The Hobbit, The Street Lawyer | 1 |
| multi_agent::249::0 | What movies, books and dishes have you recommended to me? | raw | false | Silence of the Lambs, The (1991), Tomato Sauce, James Herriot's Dog Stories | 1 |
| multi_agent::250::0 | What movies, books and dishes have you recommended to me? | raw | false | Close Shave, A (1995), North by Northwest (1959), Jelly, Donuts, My Family and Other Animals. | 1 |
| multi_agent::251::0 | What movies, books and dishes have you recommended to me? | raw | false | Pulp Fiction (1994), Babe (1995), Mushroom Risotto, Parmesan Cheese, Shipping News | 1 |
| multi_agent::252::0 | What movies, books and dishes have you recommended to me? | raw | false | Indiana Jones and the Last Crusade (1989), Salted Lassi, Anna Karenina (Oprah's Book Club) | 1 |
| multi_agent::253::0 | What movies, books and dishes have you recommended to me? | raw | false | Cool Hand Luke (1967), Maple Syrup Pancakes, A Civil Action | 1 |
| multi_agent::254::0 | What movies, books and dishes have you recommended to me? | raw | false | Red Rock West (1992), Rear Window (1954), Mushroom Risotto, Seafood, Divine Secrets of the Ya-Ya Sisterhood: A Novel | 1 |
| multi_agent::255::0 | What movies, books and dishes have you recommended to me? | raw | false | 12 Angry Men (1957), Salted Butter Toffee, Salted Caramel, Prince Caspian | 1 |
| multi_agent::256::0 | What movies, books and dishes have you recommended to me? | raw | false | City of Lost Children, The (1995), Sleeper (1973), Pretzels, Talking to Heaven: A Medium's Message of Life After Death, Embraced by the Light | 1 |
| multi_agent::257::0 | What movies, books and dishes have you recommended to me? | raw | true | All About Eve (1950), Bridge on the River Kwai, The (1957), Honey Glazed Ham, Pecan Praline, Chicken Soup for the College Soul : Inspiring and Humorous Stories for College Students (Chicken Soup for the Soul) | 1 |
| multi_agent::258::0 | What movies, books and dishes have you recommended to me? | raw | false | Terminator, The (1984), Dashi Broth, Tomato Sauce, A Walk in the Woods: Rediscovering America on the Appalachian Trail | 1 |
| multi_agent::259::0 | What movies, books and dishes have you recommended to me? | raw | false | Full Metal Jacket (1987), Good, The Bad and The Ugly, The (1966), Donuts, Tales of a Female Nomad: Living at Large in the World, Notes from a Small Island | 1 |
| multi_agent::260::0 | What movies, books and dishes have you recommended to me? | raw | false | Shallow Grave (1994), Rebecca (1940), Prosciutto and Melon, Honey Glazed Ham, Watchmen | 1 |
| multi_agent::261::0 | What movies, books and dishes have you recommended to me? | raw | true | Wings of the Dove, The (1997), Pecan Praline, The Anatomy of Motive : The FBI's Legendary Mindhunter Explores the Key to Understanding and Catching Violent Criminals, Brothel: Mustang Ranch and Its Women | 1 |
| multi_agent::262::0 | What movies, books and dishes have you recommended to me? | raw | false | Lone Star (1996), Ramen, Chicken Soup for the Christian Soul (Chicken Soup for the Soul Series (Paper)), A Second Chicken Soup for the Woman's Soul (Chicken Soup for the Soul Series) | 1 |
| multi_agent::263::0 | What movies, books and dishes have you recommended to me? | raw | false | Gandhi (1982), Balsamic Glazed Vegetables, Glazed Salmon, Crow Lake (Today Show Book Club #7), Dark Water (Mira Romantic Suspense) | 1 |
| multi_agent::264::0 | What movies, books and dishes have you recommended to me? | raw | false | Crumb (1994), Chocolate Cake, What to Expect the First Year | 1 |
| multi_agent::265::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), Jelly, Rice Krispies, The Four Agreements: A Practical Guide to Personal Freedom, The 7 Habits Of Highly Effective Teens | 1 |
| multi_agent::266::0 | What movies, books and dishes have you recommended to me? | raw | false | To Catch a Thief (1955), Honey, Candy, The Street Lawyer, Prince and the Pauper Walt Disney | 1 |
| multi_agent::267::0 | What movies, books and dishes have you recommended to me? | raw | false | Toy Story (1995), Japanese Sweet Egg Tamagoyaki, A Kitchen Witch's Cookbook, New Vegetarian: Bold and Beautiful Recipes for Every Occasion | 1 |
| multi_agent::268::0 | What movies, books and dishes have you recommended to me? | raw | false | One Flew Over the Cuckoo's Nest (1975), Fargo (1996), Brownies, Honey, Body for Life: 12 Weeks to Mental and Physical Strength | 1 |
| multi_agent::269::0 | What movies, books and dishes have you recommended to me? | raw | false | Secrets & Lies (1996), Apple Pie, Baklava, Route 66 Postcards: Greetings from the Mother Road, Tales of a Female Nomad: Living at Large in the World | 1 |
| multi_agent::270::0 | What movies, books and dishes have you recommended to me? | raw | false | To Kill a Mockingbird (1962), Killing Fields, The (1984), Chocolate Dipped Bacon, Prosciutto and Melon, The Simpsons and Philosophy: The D'oh! of Homer, The Watcher's Guide 2 (Buffy the Vampire Slayer) | 1 |
| multi_agent::271::0 | What movies, books and dishes have you recommended to me? | raw | false | Alien (1979), 39 Steps, The (1935), Salted Maple Ice Cream, Women Who Run with the Wolves | 1 |
| multi_agent::272::0 | What movies, books and dishes have you recommended to me? | raw | false | Speed (1994), Honey Glazed Ham, Woman: An Intimate Geography | 1 |
| multi_agent::273::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Honey Glazed Ham, Zen in the Art of Writing | 1 |
| multi_agent::274::0 | What movies, books and dishes have you recommended to me? | raw | false | To Catch a Thief (1955), Apple Pie, Baklava, Nickel and Dimed: On (Not) Getting By in America, Fish! A Remarkable Way to Boost Morale and Improve Results | 1 |
| multi_agent::275::0 | What movies, books and dishes have you recommended to me? | raw | false | This Is Spinal Tap (1984), Salted Lassi, Harry Potter and the Order of the Phoenix (Book 5) | 1 |
| multi_agent::276::0 | What movies, books and dishes have you recommended to me? | raw | false | Chasing Amy (1997), Chocolate Covered Pretzels, Dr. Atkins' New Diet Revolution | 1 |
| multi_agent::277::0 | What movies, books and dishes have you recommended to me? | raw | false | Ransom (1996), Taxi Driver (1976), Maple Syrup Pancakes, Dr. Atkins' New Diet Revolution | 1 |
| multi_agent::278::0 | What movies, books and dishes have you recommended to me? | raw | true | His Girl Friday (1940), Sea Salt Chocolate, Prosciutto and Melon, Even Cowgirls Get the Blues, Dark Water (Mira Romantic Suspense) | 1 |
| multi_agent::279::0 | What movies, books and dishes have you recommended to me? | raw | false | Sabrina (1954), Salted Lassi, The Man Who Mistook His Wife for a Hat: And Other Clinical Tales | 1 |
| multi_agent::280::0 | What movies, books and dishes have you recommended to me? | raw | false | Cold Comfort Farm (1995), Fruit, Rice Krispies, No Bad Dogs : The Woodhouse Way | 1 |
| multi_agent::281::0 | What movies, books and dishes have you recommended to me? | raw | false | Babe (1995), Aladdin (1992), Pork Adobo, Sweet Soy Sauce Dishes, Dude, Where's My Country? | 1 |
| multi_agent::282::0 | What movies, books and dishes have you recommended to me? | raw | false | Pink Floyd - The Wall (1982), Creme Brulee, Eats, Shoots and Leaves: The Zero Tolerance Approach to Punctuation, Zen in the Art of Writing | 1 |
| multi_agent::283::0 | What movies, books and dishes have you recommended to me? | raw | false | Ransom (1996), Ramen, HITCHHIK GD GALAXY (Hitchhiker's Trilogy (Paperback)) | 1 |
| multi_agent::284::0 | What movies, books and dishes have you recommended to me? | raw | false | Young Frankenstein (1974), Chocolate Covered Pretzels, Salted Butter Toffee, Songs of Innocence and Songs of Experience (Dover Thrift Editions) | 1 |
| multi_agent::285::0 | What movies, books and dishes have you recommended to me? | raw | false | Pocahontas (1995), Love Bug, The (1969), Tomato Sauce, Soy Sauce, The Man Who Listens to Horses | 1 |
| multi_agent::286::0 | What movies, books and dishes have you recommended to me? | raw | false | African Queen, The (1951), Grilled Portobello Mushrooms, The Source of Magic, The Grey King (The Dark is Rising Sequence) | 1 |
| multi_agent::287::0 | What movies, books and dishes have you recommended to me? | raw | false | Man with a Movie Camera (1929), Sea Salt Chocolate, 100 Best-Loved Poems (Dover Thrift Editions) | 1 |
| multi_agent::288::0 | What movies, books and dishes have you recommended to me? | raw | false | Abyss, The (1989), Salted Peanut Butter Cookies, Salted Butterscotch Pudding, The Man Who Listens to Horses, The Perfect Storm : A True Story of Men Against the Sea | 1 |
| multi_agent::289::0 | What movies, books and dishes have you recommended to me? | raw | false | Usual Suspects, The (1995), Rock, The (1996), Chocolate Cake, Nine Parts of Desire: The Hidden World of Islamic Women | 1 |
| multi_agent::290::0 | What movies, books and dishes have you recommended to me? | raw | false | Manon of the Spring (Manon des sources) (1986), Miso Soup, Gianna: Aborted... and Lived to Tell About It (Living Books), A Natural History of the Senses | 1 |
| multi_agent::291::0 | What movies, books and dishes have you recommended to me? | raw | true | Sting, The (1973), Chocolate Dipped Bacon, Salted Lassi, Crow Lake (Today Show Book Club #7) | 1 |
| multi_agent::292::0 | What movies, books and dishes have you recommended to me? | raw | false | In the Line of Fire (1993), Sea Salt Chocolate, Stupid White Men ...and Other Sorry Excuses for the State of the Nation!, The O'Reilly Factor: The Good, the Bad, and the Completely Ridiculous in American Life | 1 |
| multi_agent::293::0 | What movies, books and dishes have you recommended to me? | raw | false | Raising Arizona (1987), Rosencrantz and Guildenstern Are Dead (1990), Baklava, Seabiscuit, A Year in Provence | 1 |
| multi_agent::294::0 | What movies, books and dishes have you recommended to me? | raw | false | Forrest Gump (1994), Peanut Butter Milkshake, Rice Pudding, Civil Disobedience and Other Essays (Dover Thrift Editions) | 1 |
| multi_agent::295::0 | What movies, books and dishes have you recommended to me? | raw | false | Roman Holiday (1953), Grilled Portobello Mushrooms, Seafood, Seabiscuit: An American Legend | 1 |
| multi_agent::296::0 | What movies, books and dishes have you recommended to me? | raw | false | To Catch a Thief (1955), Banana Bread, Die HÃ?Â¤upter meiner Lieben., The Moonstone (Penguin Classics) | 1 |
| multi_agent::297::0 | What movies, books and dishes have you recommended to me? | raw | false | Much Ado About Nothing (1993), Sea Salt Chocolate, Catch Me If You Can: The True Story of a Real Fake | 1 |
| multi_agent::298::0 | What movies, books and dishes have you recommended to me? | raw | false | Heat (1995), Last of the Mohicans, The (1992), Candy, SHIPPING NEWS | 1 |
| multi_agent::299::0 | What movies, books and dishes have you recommended to me? | raw | false | Serial Mom (1994), Professional, The (1994), Baklava, Red Dwarf | 1 |
| multi_agent::300::0 | What movies, books and dishes have you recommended to me? | raw | false | Last of the Mohicans, The (1992), Alien (1979), Sweet and Sour Chicken, The Greatest Show Off Earth | 1 |
| multi_agent::301::0 | What movies, books and dishes have you recommended to me? | raw | false | Cold Comfort Farm (1995), Jelly, Lonely Planet Unpacked | 1 |
| multi_agent::302::0 | What movies, books and dishes have you recommended to me? | raw | false | Toy Story (1995), M*A*S*H (1970), Salted Caramel, Salted Lassi, Savage Inequalities: Children in America's Schools | 1 |
| multi_agent::303::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Jaws (1975), Chocolate Covered Pretzels, Lakota Woman, Bitch: In Praise of Difficult Women | 1 |
| multi_agent::304::0 | What movies, books and dishes have you recommended to me? | raw | false | Manon of the Spring (Manon des sources) (1986), Honey, Apple Pie, The Bad Beginning (A Series of Unfortunate Events, Book 1), A Wrinkle In Time | 1 |
| multi_agent::305::0 | What movies, books and dishes have you recommended to me? | raw | false | Quiet Man, The (1952), Strictly Ballroom (1992), Dashi Broth, Tomato Sauce, El Guardian Entre El Centeno, Different Seasons | 1 |
| multi_agent::306::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Honey Glazed Ham, Chocolate Dipped Bacon, Nickel and Dimed: On (Not) Getting By in America | 1 |
| multi_agent::307::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Baklava, Brownies, An Anthropologist on Mars: Seven Paradoxical Tales | 1 |
| multi_agent::308::0 | What movies, books and dishes have you recommended to me? | raw | false | Secret of Roan Inish, The (1994), Custard, Notes from a Small Island, Neither Here nor There: Travels in Europe | 1 |
| multi_agent::309::0 | What movies, books and dishes have you recommended to me? | raw | false | Shadowlands (1993), Chocolate Dipped Bacon, Salted Maple Ice Cream, Odd Girl Out: The Hidden Culture of Aggression in Girls | 1 |
| multi_agent::310::0 | What movies, books and dishes have you recommended to me? | raw | false | Crying Game, The (1992), Honey Glazed Ham, Under the Tuscan Sun: At Home in Italy | 1 |
| multi_agent::311::0 | What movies, books and dishes have you recommended to me? | raw | false | Usual Suspects, The (1995), Jelly, Uncle Shelby's ABZ Book: A Primer for Adults Only, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them | 1 |
| multi_agent::312::0 | What movies, books and dishes have you recommended to me? | raw | false | Emma (1996), Miso Soup, Parmesan Cheese, Last Chance to See | 1 |
| multi_agent::313::0 | What movies, books and dishes have you recommended to me? | raw | false | Much Ado About Nothing (1993), Salted Lassi, Salted Maple Ice Cream, Lucky Man: A Memoir | 1 |
| multi_agent::314::0 | What movies, books and dishes have you recommended to me? | raw | false | Titanic (1997), Lone Star (1996), Chocolate Dipped Bacon, Salted Maple Ice Cream, Dr. Atkins' New Diet Revolution | 1 |
| multi_agent::315::0 | What movies, books and dishes have you recommended to me? | raw | true | Diva (1981), Prosciutto and Melon, The Grey King (The Dark is Rising Sequence), The Silver Chair | 1 |
| multi_agent::316::0 | What movies, books and dishes have you recommended to me? | raw | false | The Thin Blue Line (1988), Salted Maple Ice Cream, Maple Bacon, Acqua Alta | 1 |
| multi_agent::317::0 | What movies, books and dishes have you recommended to me? | raw | false | Akira (1988), Salted Maple Ice Cream, Make the Connection: Ten Steps to a Better Body and a Better Life, 8 Weeks to Optimum Health | 1 |
| multi_agent::318::0 | What movies, books and dishes have you recommended to me? | raw | true | Dr. Strangelove or: How I Learned to Stop Worrying and Love the Bomb (1963), Salted Maple Ice Cream, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| multi_agent::319::0 | What movies, books and dishes have you recommended to me? | raw | false | Eat Drink Man Woman (1994), Salted Peanut Butter Cookies, Honey Glazed Ham, The Simpsons and Philosophy: The D'oh! of Homer, Trading Spaces Behind the Scenes: Including Decorating Tips and Tricks | 1 |
| multi_agent::320::0 | What movies, books and dishes have you recommended to me? | raw | false | George of the Jungle (1997), Apple Pie, We're Right, They're Wrong: A Handbook for Spirited Progressives, Bush at War | 1 |
| multi_agent::321::0 | What movies, books and dishes have you recommended to me? | raw | false | Wrong Trousers, The (1993), Donuts, The Woman Warrior : Memoirs of a Girlhood Among Ghosts, American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library) | 1 |
| multi_agent::322::0 | What movies, books and dishes have you recommended to me? | raw | false | Schindler's List (1993), Anchovy Pizza, Dr. Atkins' New Diet Revolution | 1 |
| multi_agent::323::0 | What movies, books and dishes have you recommended to me? | raw | false | Clerks (1994), Baklava, Pecan Pie, The Sweet Potato Queens' Book of Love, The Dance of Anger: A Woman's Guide to Changing the Patterns of Intimate Relationships | 1 |
| multi_agent::324::0 | What movies, books and dishes have you recommended to me? | raw | false | Braveheart (1995), Grilled Portobello Mushrooms, Anchovy Pizza, Enigma., Mansfield Park (Penguin Classics) | 1 |
| multi_agent::325::0 | What movies, books and dishes have you recommended to me? | raw | false | Professional, The (1994), Banana Bread, Brothel: Mustang Ranch and Its Women, The Woman Warrior : Memoirs of a Girlhood Among Ghosts | 1 |
| multi_agent::326::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Jelly, Apple Pie, The Elegant Universe: Superstrings, Hidden Dimensions, and the Quest for the Ultimate Theory, My Family and Other Animals. | 1 |
| multi_agent::327::0 | What movies, books and dishes have you recommended to me? | raw | false | Cinema Paradiso (1988), This Is Spinal Tap (1984), Japanese Curry, Glazed Salmon, GefÃ?Â¤hrliche Geliebte., The Greatest Show Off Earth | 1 |
| multi_agent::328::0 | What movies, books and dishes have you recommended to me? | raw | false | Clerks (1994), Kolya (1996), Pecan Pie, Fruit, The Curious Sofa: A Pornographic Work by Ogdred Weary | 1 |
| multi_agent::329::0 | What movies, books and dishes have you recommended to me? | raw | false | Clerks (1994), Pecan Pie, Fruit, Angela's Ashes (MMP) : A Memoir, Lucky Man: A Memoir | 1 |
| multi_agent::330::0 | What movies, books and dishes have you recommended to me? | raw | false | Boot, Das (1981), Adventures of Robin Hood, The (1938), Salted Butter Toffee, Four Major Plays: A Doll House, the Wild Duck, Hedda Gabler, the Master Builder (Signet Classics (Paperback)), Romeo and Juliet (Dover Thrift Editions) | 1 |
| multi_agent::331::0 | What movies, books and dishes have you recommended to me? | raw | false | Fugitive, The (1993), Pecan Pie, Baklava, Odd Girl Out: The Hidden Culture of Aggression in Girls, Creative Companion: How to Free Your Creative Spirit | 1 |
| multi_agent::332::0 | What movies, books and dishes have you recommended to me? | raw | false | Wizard of Oz, The (1939), Cinema Paradiso (1988), Banana Smoothie, Creme Brulee, The Portrait of a Lady (Penguin Classics), Coma (Signet Books) | 1 |
| multi_agent::333::0 | What movies, books and dishes have you recommended to me? | raw | false | Eat Drink Man Woman (1994), Chocolate Dipped Bacon, Sea Salt Chocolate, The Jane Austen Book Club, The Writing Life | 1 |
| multi_agent::334::0 | What movies, books and dishes have you recommended to me? | raw | false | GoodFellas (1990), Sea Salt Chocolate, The Dance of Anger: A Woman's Guide to Changing the Patterns of Intimate Relationships | 1 |
| multi_agent::335::0 | What movies, books and dishes have you recommended to me? | raw | false | Clerks (1994), Butch Cassidy and the Sundance Kid (1969), Chicken Stock, Stupid White Men : ...And Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::336::0 | What movies, books and dishes have you recommended to me? | raw | false | Terminator 2: Judgment Day (1991), Custard, The Importance of Being Earnest (Dover Thrift Editions) | 1 |
| multi_agent::337::0 | What movies, books and dishes have you recommended to me? | raw | false | Mr. Smith Goes to Washington (1939), Jelly, Custard, The Scarlet Letter: A Romance (The Penguin American Library) | 1 |
| multi_agent::338::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Three Colors: Blue (1993), Honey, ANGELA'S ASHES | 2 |
| multi_agent::339::0 | What movies, books and dishes have you recommended to me? | raw | false | Jurassic Park (1993), Prosciutto and Melon, Civil Disobedience and Other Essays (Dover Thrift Editions) | 2 |
| multi_agent::340::0 | What movies, books and dishes have you recommended to me? | raw | false | Man Who Would Be King, The (1975), Men in Black (1997), Buffalo Wings, Salsa, ALL MY PATIENTS ARE UNDER THE BED, James Herriot's Dog Stories | 1 |
| multi_agent::341::0 | What movies, books and dishes have you recommended to me? | raw | false | Apt Pupil (1998), Soy Sauce, The Snow Leopard (Penguin Nature Classics), A Sand County Almanac (Outdoor Essays & Reflections) | 1 |
| multi_agent::342::0 | What movies, books and dishes have you recommended to me? | raw | false | Mr. Smith Goes to Washington (1939), Empire Strikes Back, The (1980), Sweet and Sour Chicken, Woman: An Intimate Geography | 1 |
| multi_agent::343::0 | What movies, books and dishes have you recommended to me? | raw | false | Manhattan (1979), Chocolate Dipped Bacon, Pecan Praline, Parliament of Whores: A Lone Humorist Attempts to Explain the Entire U.S. Government, 9-11 | 1 |
| multi_agent::344::0 | What movies, books and dishes have you recommended to me? | raw | false | Graduate, The (1967), Sabrina (1954), Salted Peanut Butter Cookies, The Te of Piglet | 1 |
| multi_agent::345::0 | What movies, books and dishes have you recommended to me? | raw | false | Singin' in the Rain (1952), Salted Maple Ice Cream, Chocolate Covered Pretzels, New Vegetarian: Bold and Beautiful Recipes for Every Occasion | 1 |
| multi_agent::346::0 | What movies, books and dishes have you recommended to me? | raw | false | Quiet Man, The (1952), Salted Lassi, Salted Maple Ice Cream, The Nitpicker's Guide for Next Generation Trekkers, Vol. 2, Mike Nelson's Movie Megacheese | 1 |
| multi_agent::347::0 | What movies, books and dishes have you recommended to me? | raw | false | Usual Suspects, The (1995), Chocolate Covered Pretzels, Ghost World | 1 |
| multi_agent::348::0 | What movies, books and dishes have you recommended to me? | raw | false | Arsenic and Old Lace (1944), Chocolate Dipped Bacon, Salted Caramel, El Principito | 1 |
| multi_agent::349::0 | What movies, books and dishes have you recommended to me? | raw | true | L.A. Confidential (1997), Salted Lassi, Salted Maple Ice Cream, Book of Virtues | 1 |
| multi_agent::350::0 | What movies, books and dishes have you recommended to me? | raw | false | Toy Story (1995), Back to the Future (1985), Pecan Praline, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them, Book of Virtues | 1 |
| multi_agent::351::0 | What movies, books and dishes have you recommended to me? | raw | false | Perfect World, A (1993), Orange Chicken, Talking to Heaven: A Medium's Message of Life After Death, Chicken Soup for the Soul at Work (Chicken Soup for the Soul Series (Paper)) | 1 |
| multi_agent::352::0 | What movies, books and dishes have you recommended to me? | raw | false | Sleeper (1973), Clockwork Orange, A (1971), Chocolate Dipped Bacon, Scientific Progress Goes 'Boink':  A Calvin and Hobbes Collection | 1 |
| multi_agent::353::0 | What movies, books and dishes have you recommended to me? | raw | false | Babe (1995), Full Monty, The (1997), Salted Caramel, To Ride a Silver Broomstick: New Generation Witchcraft | 1 |
| multi_agent::354::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Maple Bacon, Pecan Praline, Different Seasons | 1 |
| multi_agent::355::0 | What movies, books and dishes have you recommended to me? | raw | false | Face/Off (1997), Rice Krispies, To Ride a Silver Broomstick: New Generation Witchcraft | 1 |
| multi_agent::356::0 | What movies, books and dishes have you recommended to me? | raw | false | Roman Holiday (1953), Arsenic and Old Lace (1944), Salted Caramel, Salted Butterscotch Pudding, Awakening the Buddha Within : Tibetan Wisdom for the Western World | 1 |
| multi_agent::357::0 | What movies, books and dishes have you recommended to me? | raw | false | Patton (1970), Hot Shots! Part Deux (1993), Banana Bread, Civil Disobedience and Other Essays (Dover Thrift Editions), High Tide in Tucson : Essays from Now or Never | 1 |
| multi_agent::358::0 | What movies, books and dishes have you recommended to me? | raw | false | Men in Black (1997), Godfather, The (1972), Sea Salt Chocolate, Salted Butter Toffee, A Civil Action, Dead Man Walking: An Eyewitness Account of the Death Penalty in the United States | 1 |
| multi_agent::359::0 | What movies, books and dishes have you recommended to me? | raw | false | Diva (1981), Banana Bread, Donuts, Chicken Soup for the Soul (Chicken Soup for the Soul), The Book of Questions | 1 |
| multi_agent::360::0 | What movies, books and dishes have you recommended to me? | raw | false | Braveheart (1995), Get Shorty (1995), Salted Peanut Butter Cookies, 100 Selected Poems by E. E. Cummings | 1 |
| multi_agent::361::0 | What movies, books and dishes have you recommended to me? | raw | false | Lion King, The (1994), Chocolate Dipped Bacon, Body for Life: 12 Weeks to Mental and Physical Strength, Prescription for Nutritional Healing: A Practical A-Z Reference to Drug-Free Remedies Using Vitamins, Minerals, Herbs & Food Supplements | 1 |
| multi_agent::362::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Aged Cheddar, Tales of a Female Nomad: Living at Large in the World, Neither Here nor There: Travels in Europe | 1 |
| multi_agent::363::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Good Will Hunting (1997), Salted Butterscotch Pudding, Salted Butter Toffee, The Snow Leopard (Penguin Nature Classics) | 1 |
| multi_agent::364::0 | What movies, books and dishes have you recommended to me? | raw | false | Strictly Ballroom (1992), Dumplings with Soy Dip, Seaweed Salad, The Case for Christ:  A Journalist's Personal Investigation of the Evidence for Jesus, The Prayer of Jabez: Breaking Through to the Blessed Life | 1 |
| multi_agent::365::0 | What movies, books and dishes have you recommended to me? | raw | false | Some Kind of Wonderful (1987), Pecan Praline, Prosciutto and Melon, Under the Tuscan Sun, Notes from a Small Island | 1 |
| multi_agent::366::0 | What movies, books and dishes have you recommended to me? | raw | false | Apartment, The (1960), Soy Sauce, Downsize This! Random Threats from an Unarmed American, We're Right, They're Wrong: A Handbook for Spirited Progressives | 1 |
| multi_agent::367::0 | What movies, books and dishes have you recommended to me? | raw | false | Monty Python and the Holy Grail (1974), Salted Butter Toffee, Prosciutto and Melon, Good in Bed | 1 |
| multi_agent::368::0 | What movies, books and dishes have you recommended to me? | raw | false | Fifth Element, The (1997), Anchovy Pizza, SEVEN HABITS OF HIGHLY EFFECTIVE PEOPLE : Powerful Lessons in Personal Change | 1 |
| multi_agent::369::0 | What movies, books and dishes have you recommended to me? | raw | false | Four Weddings and a Funeral (1994), Chocolate Cake, Small Sacrifices: A True Story of Passion and Murder | 1 |
| multi_agent::370::0 | What movies, books and dishes have you recommended to me? | raw | false | Seven (Se7en) (1995), Jelly, Who Moved My Cheese? An Amazing Way to Deal with Change in Your Work and in Your Life | 1 |
| multi_agent::371::0 | What movies, books and dishes have you recommended to me? | raw | false | Crumb (1994), Looking for Richard (1996), Soy Sauce, Ramen, Where the Sidewalk Ends : Poems and Drawings | 1 |
| multi_agent::372::0 | What movies, books and dishes have you recommended to me? | raw | false | Men in Black (1997), Mission: Impossible (1996), Sea Salt Chocolate, Lust for Life | 1 |
| multi_agent::373::0 | What movies, books and dishes have you recommended to me? | raw | false | Sense and Sensibility (1995), Chocolate Covered Pretzels, The South Beach Diet: The Delicious, Doctor-Designed, Foolproof Plan for Fast and Healthy Weight Loss | 1 |
| multi_agent::374::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Salted Lassi, The Essential 55: An Award-Winning Educator's Rules for Discovering the Successful Student in Every Child | 1 |
| multi_agent::375::0 | What movies, books and dishes have you recommended to me? | raw | false | Supercop (1992), Terminator 2: Judgment Day (1991), Spicy Ramen, Thai Green Curry, The Street Lawyer | 1 |
| multi_agent::376::0 | What movies, books and dishes have you recommended to me? | raw | false | Secrets & Lies (1996), Hamlet (1996), Candy, The Screwtape Letters | 1 |
| multi_agent::377::0 | What movies, books and dishes have you recommended to me? | raw | false | Die Hard (1988), Pecan Pie, Jelly, Last Chance to See | 1 |
| multi_agent::378::0 | What movies, books and dishes have you recommended to me? | raw | false | High Noon (1952), Fruit, A Mind of Its Own: A Cultural History of the Penis | 1 |
| multi_agent::379::0 | What movies, books and dishes have you recommended to me? | raw | false | Speed (1994), Donuts, Candy, Divine Secrets of the Ya-Ya Sisterhood: A Novel | 1 |
| multi_agent::380::0 | What movies, books and dishes have you recommended to me? | raw | false | Good, The Bad and The Ugly, The (1966), High Noon (1952), Salted Butter Toffee, Salted Peanut Butter Cookies, The Prophet | 1 |
| multi_agent::381::0 | What movies, books and dishes have you recommended to me? | raw | false | Stand by Me (1986), Cinema Paradiso (1988), Cajun Shrimp, A Civil Action | 1 |
| multi_agent::382::0 | What movies, books and dishes have you recommended to me? | raw | false | Last Man Standing (1996), Tombstone (1993), Fruit, SEAT OF THE SOUL | 1 |
| multi_agent::383::0 | What movies, books and dishes have you recommended to me? | raw | false | Roman Holiday (1953), Chocolate Dipped Bacon, Salted Maple Ice Cream, Welcome to the World Baby Girl | 1 |
| multi_agent::384::0 | What movies, books and dishes have you recommended to me? | raw | false | All About Eve (1950), Jean de Florette (1986), Jelly, HITCHHIK GD GALAXY (Hitchhiker's Trilogy (Paperback)), More Than Complete Hitchhiker's Guide | 1 |
| multi_agent::385::0 | What movies, books and dishes have you recommended to me? | raw | false | In the Line of Fire (1993), Tomato Sauce, Ramen, Ex Libris : Confessions of a Common Reader | 1 |
| multi_agent::386::0 | What movies, books and dishes have you recommended to me? | raw | false | Blade Runner (1982), Chocolate Dipped Bacon, Left Behind: A Novel of the Earth's Last Days (Left Behind #1) | 1 |
| multi_agent::387::0 | What movies, books and dishes have you recommended to me? | raw | false | To Catch a Thief (1955), It Happened One Night (1934), Sea Salt Chocolate, Das Hotel New Hampshire | 1 |
| multi_agent::388::0 | What movies, books and dishes have you recommended to me? | raw | false | Starship Troopers (1997), Stargate (1994), Chocolate Covered Pretzels, Flu: The Story of the Great Influenza Pandemic of 1918 and the Search for the Virus That Caused It, The Hot Zone | 1 |
| multi_agent::389::0 | What movies, books and dishes have you recommended to me? | raw | false | Starship Troopers (1997), Star Trek: The Motion Picture (1979), Maple Syrup Pancakes, Angela's Ashes: A Memoir | 1 |
| multi_agent::390::0 | What movies, books and dishes have you recommended to me? | raw | false | Henry V (1989), Chocolate Covered Pretzels, A Night Without Armor : Poems | 1 |
| multi_agent::391::0 | What movies, books and dishes have you recommended to me? | raw | false | Forrest Gump (1994), Princess Bride, The (1987), Spicy Hotpot, Harry Potter and the Goblet of Fire (Book 4) | 1 |
| multi_agent::392::0 | What movies, books and dishes have you recommended to me? | raw | false | Heat (1995), Mushroom Risotto, Anna Karenina (Oprah's Book Club), Love in the Time of Cholera (Penguin Great Books of the 20th Century) | 1 |
| multi_agent::393::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Beef Stew, Mars and Venus on a Date : A Guide to Navigating the 5 Stages of Dating to Create a Loving and Lasting Relationship | 1 |
| multi_agent::394::0 | What movies, books and dishes have you recommended to me? | raw | false | Men in Black (1997), Seafood, Savage Inequalities: Children in America's Schools | 1 |
| multi_agent::395::0 | What movies, books and dishes have you recommended to me? | raw | false | Primal Fear (1996), Maple Syrup Pancakes, Pecan Pie, Bush at War | 1 |
| multi_agent::396::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Chocolate Covered Pretzels, Salted Lassi, The Perfect Storm : A True Story of Men Against the Sea, The Man Who Listens to Horses | 1 |
| multi_agent::397::0 | What movies, books and dishes have you recommended to me? | raw | false | English Patient, The (1996), Fruit, Walden and Other Writings, Small Wonder: Essays | 1 |
| multi_agent::398::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Borscht Soup, 9-11, A Royal Duty | 1 |
| multi_agent::399::0 | What movies, books and dishes have you recommended to me? | raw | false | Kolya (1996), Sabrina (1954), Anchovy Pizza, Ramen, The Da Vinci Code | 1 |
| multi_agent::400::0 | What movies, books and dishes have you recommended to me? | raw | false | Graduate, The (1967), Sea Salt Chocolate, Rosencrantz & Guildenstern Are Dead | 1 |
| multi_agent::401::0 | What movies, books and dishes have you recommended to me? | raw | false | Crimson Tide (1995), Pecan Praline, Tommo & Hawk | 1 |
| multi_agent::402::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Jelly, Banana Bread, Watchmen, Chobits (Chobits) | 1 |
| multi_agent::403::0 | What movies, books and dishes have you recommended to me? | raw | false | Delicatessen (1991), Chocolate Covered Pretzels, Left Behind: A Novel of the Earth's Last Days (Left Behind #1) | 1 |
| multi_agent::404::0 | What movies, books and dishes have you recommended to me? | raw | false | Jerry Maguire (1996), Aged Cheddar, A Wrinkle In Time, Harry Potter and the Goblet of Fire (Book 4) | 1 |
| multi_agent::405::0 | What movies, books and dishes have you recommended to me? | raw | false | Raising Arizona (1987), Princess Bride, The (1987), Pecan Praline, Chicken Soup for the Soul at Work (Chicken Soup for the Soul Series (Paper)), Peace Is Every Step: The Path of Mindfulness in Everyday Life | 1 |
| multi_agent::406::0 | What movies, books and dishes have you recommended to me? | raw | false | True Lies (1994), Chicken Stock, Aged Cheddar, Cosmos | 1 |
| multi_agent::407::0 | What movies, books and dishes have you recommended to me? | raw | false | Chasing Amy (1997), Sweet and Sour Chicken, More Than Complete Hitchhiker's Guide | 1 |
| multi_agent::408::0 | What movies, books and dishes have you recommended to me? | raw | false | Apt Pupil (1998), Salted Peanut Butter Cookies, Honey Glazed Ham, Voyage on the Great Titanic: The Diary of Margaret Ann Brady (Dear America) | 1 |
| multi_agent::409::0 | What movies, books and dishes have you recommended to me? | raw | false | Henry V (1989), Chocolate Covered Pretzels, Lust for Life | 1 |
| multi_agent::410::0 | What movies, books and dishes have you recommended to me? | raw | false | Last Man Standing (1996), Salted Butter Toffee, A Dangerous Fortune, The Mists of Avalon | 1 |
| multi_agent::411::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather: Part II, The (1974), Chocolate Covered Pretzels, Death of A Salesman, Romeo and Juliet (Bantam Classic) | 1 |
| multi_agent::412::0 | What movies, books and dishes have you recommended to me? | raw | false | Sling Blade (1996), Chocolate Covered Pretzels, Harry Potter and the Goblet of Fire (Book 4), Harry Potter and the Goblet of Fire (Book 4) | 1 |
| multi_agent::413::0 | What movies, books and dishes have you recommended to me? | raw | false | Man with a Movie Camera (1929), Hoop Dreams (1994), Chocolate Dipped Bacon, Pecan Praline, El Principito | 1 |
| multi_agent::414::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Baklava, The Girlfriends' Guide to Pregnancy | 1 |
| multi_agent::415::0 | What movies, books and dishes have you recommended to me? | raw | false | Gattaca (1997), Banana Smoothie, Cheesecake, In the Heart of the Sea: The Tragedy of the Whaleship Essex | 1 |
| multi_agent::416::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Chocolate Covered Pretzels, The Cases That Haunt Us | 1 |
| multi_agent::417::0 | What movies, books and dishes have you recommended to me? | raw | false | Some Kind of Wonderful (1987), Rice Krispies, Gianna: Aborted... and Lived to Tell About It (Living Books) | 1 |
| multi_agent::418::0 | What movies, books and dishes have you recommended to me? | raw | true | Wings of Desire (1987), Milk Tea, Vanilla Milkshake, Parliament of Whores: A Lone Humorist Attempts to Explain the Entire U.S. Government, Stupid White Men ...and Other Sorry Excuses for the State of the Nation! | 1 |
| multi_agent::419::0 | What movies, books and dishes have you recommended to me? | raw | false | Terminator, The (1984), Salted Butterscotch Pudding, 10 Lb. Penalty | 1 |
| multi_agent::420::0 | What movies, books and dishes have you recommended to me? | raw | false | Fly Away Home (1996), Salted Butter Toffee, Salted Butterscotch Pudding, A Civil Action | 1 |
| multi_agent::421::0 | What movies, books and dishes have you recommended to me? | raw | false | Eat Drink Man Woman (1994), Raging Bull (1980), Baklava, Chicken Soup for the Christian Soul (Chicken Soup for the Soul Series (Paper)), The Purpose-Driven Life: What on Earth Am I Here For? | 1 |
| multi_agent::422::0 | What movies, books and dishes have you recommended to me? | raw | false | Third Man, The (1949), Rice Krispies, The Kiss, The Sweet Potato Queens' Book of Love | 1 |
| multi_agent::423::0 | What movies, books and dishes have you recommended to me? | raw | false | Eat Drink Man Woman (1994), Chocolate Covered Pretzels, Ain't I A Woman!: A Book of Women's Poetry from Around the World | 1 |
| multi_agent::424::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek III: The Search for Spock (1984), Day the Earth Stood Still, The (1951), Donuts, Baklava, Chicken Soup for the Christian Soul (Chicken Soup for the Soul Series (Paper)), Nine Parts of Desire: The Hidden World of Islamic Women | 1 |
| multi_agent::425::0 | What movies, books and dishes have you recommended to me? | raw | false | 12 Angry Men (1957), Eat Drink Man Woman (1994), Donuts, Divine Secrets of the Ya-Ya Sisterhood: A Novel, Snow Falling on Cedars | 1 |
| multi_agent::426::0 | What movies, books and dishes have you recommended to me? | raw | false | Pulp Fiction (1994), Beef Stew, The Portrait of a Lady (Penguin Classics), Nobilta. Commissario Brunettis siebter Fall. | 1 |
| multi_agent::427::0 | What movies, books and dishes have you recommended to me? | raw | false | Crumb (1994), Salted Butterscotch Pudding, Amusing Ourselves to Death: Public Discourse in the Age of Show Business | 1 |
| multi_agent::428::0 | What movies, books and dishes have you recommended to me? | raw | false | Raging Bull (1980), Salted Butter Toffee, Honey Glazed Ham, The Silver Chair | 1 |
| multi_agent::429::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Braveheart (1995), Parmesan Cheese, The Tao of Pooh | 1 |
| multi_agent::430::0 | What movies, books and dishes have you recommended to me? | raw | false | Princess Bride, The (1987), Pecan Praline, Chocolate Dipped Bacon, Behind the Scenes at the Museum | 1 |
| multi_agent::431::0 | What movies, books and dishes have you recommended to me? | raw | false | Young Frankenstein (1974), Prosciutto and Melon, Lakota Woman, The Woman Warrior : Memoirs of a Girlhood Among Ghosts | 1 |
| multi_agent::432::0 | What movies, books and dishes have you recommended to me? | raw | false | English Patient, The (1996), Soy Sauce, MY SWEET AUDRINA, Das Hotel New Hampshire | 1 |
| multi_agent::433::0 | What movies, books and dishes have you recommended to me? | raw | false | Braveheart (1995), Anchovy Pizza, The Blue Day Book | 1 |
| multi_agent::434::0 | What movies, books and dishes have you recommended to me? | raw | false | Jurassic Park (1993), Speed (1994), Salted Butter Toffee, Seabiscuit: An American Legend | 1 |
| multi_agent::435::0 | What movies, books and dishes have you recommended to me? | raw | false | Raising Arizona (1987), Butch Cassidy and the Sundance Kid (1969), Prosciutto and Melon, Chocolate Dipped Bacon, The Fellowship of the Ring | 1 |
| multi_agent::436::0 | What movies, books and dishes have you recommended to me? | raw | false | Supercop (1992), Empire Strikes Back, The (1980), Salted Lassi, Angels and Demons | 1 |
| multi_agent::437::0 | What movies, books and dishes have you recommended to me? | raw | false | Chasing Amy (1997), Groundhog Day (1993), Salted Peanut Butter Cookies, Foundations Edge, Red Dwarf | 1 |
| multi_agent::438::0 | What movies, books and dishes have you recommended to me? | raw | true | Stand by Me (1986), Rice Krispies, Savage Inequalities: Children in America's Schools | 1 |
| multi_agent::439::0 | What movies, books and dishes have you recommended to me? | raw | false | Shawshank Redemption, The (1994), Apple Pie, Mindhunter : Inside the FBI's Elite Serial Crime Unit, Empty Promises | 1 |
| multi_agent::440::0 | What movies, books and dishes have you recommended to me? | raw | false | When Harry Met Sally... (1989), Brownies, Savage Inequalities: Children in America's Schools | 1 |
| multi_agent::441::0 | What movies, books and dishes have you recommended to me? | raw | false | Hamlet (1996), Honey, Custard, The Demon-Haunted World: Science As a Candle in the Dark | 1 |
| multi_agent::442::0 | What movies, books and dishes have you recommended to me? | raw | false | 2001: A Space Odyssey (1968), Men in Black (1997), BBQ Ribs, The Darwin Awards: Evolution in Action, A Walk in the Woods: Rediscovering America on the Appalachian Trail | 1 |
| multi_agent::443::0 | What movies, books and dishes have you recommended to me? | raw | false | Koyaanisqatsi (1983), Rice Krispies, Candy, Book of Tea, The Te of Piglet | 1 |
| multi_agent::444::0 | What movies, books and dishes have you recommended to me? | raw | false | One Flew Over the Cuckoo's Nest (1975), Braveheart (1995), Honey Glazed Ham, Salted Butter Toffee, The Fellowship of the Ring, El Senor De Los Anillos: El Retorno Del Rey (Tolkien, J. R. R. Lord of the Rings. 3.) | 1 |
| multi_agent::445::0 | What movies, books and dishes have you recommended to me? | raw | false | Schindler's List (1993), Dashi Broth, LET ME CALL YOU SWEETHEART | 1 |
| multi_agent::446::0 | What movies, books and dishes have you recommended to me? | raw | false | Three Colors: Blue (1993), Tamarind Candy, Purple Cow: Transform Your Business by Being Remarkable | 1 |
| multi_agent::447::0 | What movies, books and dishes have you recommended to me? | raw | false | One Flew Over the Cuckoo's Nest (1975), Anchovy Pizza, What to Expect When You're Expecting (Revised Edition) | 1 |
| multi_agent::448::0 | What movies, books and dishes have you recommended to me? | raw | false | Roman Holiday (1953), Young Frankenstein (1974), Salted Peanut Butter Cookies, Welcome to the World Baby Girl, GefÃ?Â¤hrliche Geliebte. | 1 |
| multi_agent::449::0 | What movies, books and dishes have you recommended to me? | raw | false | Rosencrantz and Guildenstern Are Dead (1990), Chocolate Dipped Bacon, Harry Potter and the Goblet of Fire (Book 4) | 1 |
| multi_agent::450::0 | What movies, books and dishes have you recommended to me? | raw | false | Harold and Maude (1971), Apple Pie, Orfe | 1 |
| multi_agent::451::0 | What movies, books and dishes have you recommended to me? | raw | false | Schindler's List (1993), Secrets & Lies (1996), Chocolate Dipped Bacon, Honey Glazed Ham, The Man Who Mistook His Wife for a Hat: And Other Clinical Tales | 1 |
| multi_agent::452::0 | What movies, books and dishes have you recommended to me? | raw | false | Like Water For Chocolate (Como agua para chocolate) (1992), Salted Lassi, The Tao of Pooh, The Dilbert Principle: A Cubicle'S-Eye View of Bosses, Meetings, Management Fads & Other Workplace Afflictions | 1 |
| multi_agent::453::0 | What movies, books and dishes have you recommended to me? | raw | false | Shallow Grave (1994), Japanese Curry, Hoisin Glazed Duck, Self Matters : Creating Your Life from the Inside Out | 1 |
| multi_agent::454::0 | What movies, books and dishes have you recommended to me? | raw | false | Bound (1996), Shine (1996), Salted Caramel, Honey Glazed Ham, The Case for Christ:  A Journalist's Personal Investigation of the Evidence for Jesus, Their eyes were watching God: A novel | 1 |
| multi_agent::455::0 | What movies, books and dishes have you recommended to me? | raw | false | Boot, Das (1981), Ben-Hur (1959), Prosciutto and Melon, Chicken Soup for the Pet Lover's Soul (Chicken Soup for the Soul) | 1 |
| multi_agent::456::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Wars (1977), Apple Pie, Brownies, Good Faeries Bad Faeries | 1 |
| multi_agent::457::0 | What movies, books and dishes have you recommended to me? | raw | false | Groundhog Day (1993), Cyrano de Bergerac (1990), Candy, American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library) | 1 |
| multi_agent::458::0 | What movies, books and dishes have you recommended to me? | raw | false | Adventures of Robin Hood, The (1938), Ramen, Nickel and Dimed: On (Not) Getting By in America, Purple Cow: Transform Your Business by Being Remarkable | 1 |
| multi_agent::459::0 | What movies, books and dishes have you recommended to me? | raw | false | Mrs. Brown (Her Majesty, Mrs. Brown) (1997), Postino, Il (1994), Sweet and Sour Shrimp, Borscht Soup, 10 Lb. Penalty | 1 |
| multi_agent::460::0 | What movies, books and dishes have you recommended to me? | raw | false | Raising Arizona (1987), Chocolate Covered Pretzels, Savage Inequalities: Children in America's Schools | 1 |
| multi_agent::461::0 | What movies, books and dishes have you recommended to me? | raw | false | Graduate, The (1967), Soy Sauce, Creative Companion: How to Free Your Creative Spirit | 1 |
| multi_agent::462::0 | What movies, books and dishes have you recommended to me? | raw | false | Winnie the Pooh and the Blustery Day (1968), Close Shave, A (1995), Sea Salt Chocolate, The Universe in a Nutshell, A Brief History of Time : The Updated and Expanded Tenth Anniversary Edition | 1 |
| multi_agent::463::0 | What movies, books and dishes have you recommended to me? | raw | false | Menace II Society (1993), Salted Butterscotch Pudding, The Coming Plague: Newly Emerging Diseases in a World Out of Balance | 1 |
| multi_agent::464::0 | What movies, books and dishes have you recommended to me? | raw | false | Fargo (1996), Brownies, EVERYTHING SHE EVER WANTED | 1 |
| multi_agent::465::0 | What movies, books and dishes have you recommended to me? | raw | false | Pulp Fiction (1994), Empire Strikes Back, The (1980), Chocolate Cake, Sense and Sensibility (World's Classics), Shipping News | 1 |
| multi_agent::466::0 | What movies, books and dishes have you recommended to me? | raw | true | Young Frankenstein (1974), Spicy Szechuan Tofu, Die Gefahrten I | 1 |
| multi_agent::467::0 | What movies, books and dishes have you recommended to me? | raw | false | Last of the Mohicans, The (1992), Salted Lassi, Salted Butterscotch Pudding, Divine Secrets of the Ya-Ya Sisterhood: A Novel | 1 |
| multi_agent::468::0 | What movies, books and dishes have you recommended to me? | raw | false | Boot, Das (1981), Cyrano de Bergerac (1990), Honey Garlic Chicken, Bibliotherapy: The Girl's Guide to Books for Every Phase of Our Lives, The Writing Life | 1 |
| multi_agent::469::0 | What movies, books and dishes have you recommended to me? | raw | false | Star Trek: The Wrath of Khan (1982), Salted Butter Toffee, Honey Glazed Ham, The Man Who Listens to Horses | 1 |
| multi_agent::470::0 | What movies, books and dishes have you recommended to me? | raw | false | Raise the Red Lantern (1991), Good Will Hunting (1997), Chocolate Dipped Bacon, Flow: The Psychology of Optimal Experience | 1 |
| multi_agent::471::0 | What movies, books and dishes have you recommended to me? | raw | true | Raise the Red Lantern (1991), Chocolate Covered Pretzels, Sea Salt Chocolate, Odd Girl Out: The Hidden Culture of Aggression in Girls, Man's Search for Meaning: An Introduction to Logotherapy | 1 |
| multi_agent::472::0 | What movies, books and dishes have you recommended to me? | raw | false | Rebecca (1940), Air Force One (1997), Tomato Sauce, Sonnets from the Portuguese and Other Poems (Dover Thrift Editions), Selected Poems (Dover Thrift Edition) | 1 |
| multi_agent::473::0 | What movies, books and dishes have you recommended to me? | raw | false | Hamlet (1996), Sea Salt Chocolate, Honey Glazed Ham, The Demon-Haunted World: Science As a Candle in the Dark | 1 |
| multi_agent::474::0 | What movies, books and dishes have you recommended to me? | raw | true | As Good As It Gets (1997), Chocolate Dipped Bacon, Under the Tuscan Sun, McCarthy's Bar: A Journey of Discovery In Ireland | 1 |
| multi_agent::475::0 | What movies, books and dishes have you recommended to me? | raw | false | Wyatt Earp (1994), Mushroom Risotto, Politically Correct Bedtime Stories: Modern Tales for Our Life and Times | 1 |
| multi_agent::476::0 | What movies, books and dishes have you recommended to me? | raw | false | Cyrano de Bergerac (1990), Sense and Sensibility (1995), Fruit, In the Kitchen With Rosie: Oprah's Favorite Recipes | 1 |
| multi_agent::477::0 | What movies, books and dishes have you recommended to me? | raw | false | Apartment, The (1960), Miso Soup, Bibliotherapy: The Girl's Guide to Books for Every Phase of Our Lives | 1 |
| multi_agent::478::0 | What movies, books and dishes have you recommended to me? | raw | false | Godfather, The (1972), Pecan Pie, A Natural History of the Senses, Mars and Venus on a Date : A Guide to Navigating the 5 Stages of Dating to Create a Loving and Lasting Relationship | 1 |
| multi_agent::479::0 | What movies, books and dishes have you recommended to me? | raw | false | Interview with the Vampire (1994), Sweet and Sour Pork, One L : The Turbulent True Story of a First Year at Harvard Law School | 1 |
| multi_agent::480::0 | What movies, books and dishes have you recommended to me? | raw | false | Local Hero (1983), Sea Salt Chocolate, Salted Butterscotch Pudding, Talking to Heaven: A Medium's Message of Life After Death | 1 |
| multi_agent::481::0 | What movies, books and dishes have you recommended to me? | raw | false | Young Guns (1988), Last Man Standing (1996), Salted Peanut Butter Cookies, Chocolate Dipped Bacon, Lust for Life, Why Cats Paint: A Theory of Feline Aesthetics | 1 |
| multi_agent::482::0 | What movies, books and dishes have you recommended to me? | raw | false | Return of the Jedi (1983), Godfather, The (1972), Salted Maple Ice Cream, The Fellowship of the Ring (The Lord of the Rings, Part 1) | 1 |
| multi_agent::483::0 | What movies, books and dishes have you recommended to me? | raw | false | Parent Trap, The (1961), Maple Bacon, Salted Maple Ice Cream, All Through The Night : A Suspense Story | 1 |
| multi_agent::484::0 | What movies, books and dishes have you recommended to me? | raw | false | Citizen Kane (1941), Salted Maple Ice Cream, The Man Who Mistook His Wife for a Hat: And Other Clinical Tales | 1 |
| multi_agent::485::0 | What movies, books and dishes have you recommended to me? | raw | false | Some Like It Hot (1959), Candy, El Guardian Entre El Centeno | 1 |
| multi_agent::486::0 | What movies, books and dishes have you recommended to me? | raw | false | Akira (1988), Salted Caramel, Guns, Germs, and Steel: The Fates of Human Societies | 1 |
| multi_agent::487::0 | What movies, books and dishes have you recommended to me? | raw | false | Hunt for Red October, The (1990), Sea Salt Chocolate, Chocolate Covered Pretzels, In the Heart of the Sea: The Tragedy of the Whaleship Essex, Midnight in the Garden of Good and Evil: A Savannah Story | 1 |
| multi_agent::488::0 | What movies, books and dishes have you recommended to me? | raw | false | Roman Holiday (1953), Chasing Amy (1997), Salted Butterscotch Pudding, Salted Lassi, Book of Virtues | 1 |
| multi_agent::489::0 | What movies, books and dishes have you recommended to me? | raw | false | Rosencrantz and Guildenstern Are Dead (1990), Aladdin (1992), Aged Cheddar, Chobits Vol.1 | 1 |
| multi_agent::490::0 | What movies, books and dishes have you recommended to me? | raw | false | Air Force One (1997), Salted Peanut Butter Cookies, Chocolate Dipped Bacon, Hamlet (Bantam Classics) | 1 |
| multi_agent::491::0 | What movies, books and dishes have you recommended to me? | raw | false | Maverick (1994), Chocolate Dipped Bacon, The Psychologist's Book of Self-Tests: 25 Love, Sex, Intelligence, Career, and Personality Tests Developed by Professionals to Reveal the Real You | 1 |
| multi_agent::492::0 | What movies, books and dishes have you recommended to me? | raw | false | Ben-Hur (1959), Chocolate Covered Pretzels, Salted Butter Toffee, Dr. Atkins' New Diet Revolution | 1 |
| multi_agent::493::0 | What movies, books and dishes have you recommended to me? | raw | false | Great Escape, The (1963), Chocolate Covered Pretzels, The Clan of the Cave Bear : a novel | 1 |
| multi_agent::494::0 | What movies, books and dishes have you recommended to me? | raw | false | Jean de Florette (1986), Mr. Smith Goes to Washington (1939), Chocolate Dipped Bacon, Prosciutto and Melon, The Freedom Writers Diary : How a Teacher and 150 Teens Used Writing to Change Themselves and the World Around Them, Book of Virtues | 1 |
| multi_agent::495::0 | What movies, books and dishes have you recommended to me? | raw | false | As Good As It Gets (1997), Wizard of Oz, The (1939), Salted Lassi, Pecan Praline, Mars and Venus on a Date : A Guide to Navigating the 5 Stages of Dating to Create a Loving and Lasting Relationship | 1 |
| multi_agent::496::0 | What movies, books and dishes have you recommended to me? | raw | false | Christmas Carol, A (1938), Custard, Candy, Why Cats Paint: A Theory of Feline Aesthetics | 1 |
| multi_agent::497::0 | What movies, books and dishes have you recommended to me? | raw | false | Gone with the Wind (1939), Cyrano de Bergerac (1990), Salted Lassi, Maple Bacon, American Indian Myths and Legends (Pantheon Fairy Tale and Folklore Library), There Are No Children Here: The Story of Two Boys Growing Up in the Other America | 1 |
| multi_agent::498::0 | What movies, books and dishes have you recommended to me? | raw | false | Wings of the Dove, The (1997), Maple Bacon, Bitter Harvest | 1 |
| multi_agent::499::0 | What movies, books and dishes have you recommended to me? | raw | false | Raging Bull (1980), As Good As It Gets (1997), Salted Maple Ice Cream, Salted Peanut Butter Cookies, What to Expect When You're Expecting (Revised Edition), Your Pregnancy: Week by Week (Your Pregnancy Series) | 1 |
| roles::0::0 | What are the main responsibilities of a person born on August 23rd? | raw | true | Handle financial transactions and serve clients | 4 |
| roles::1::0 | What email address suffix do people with a high school education typically use? | raw | true | @pioneerconstructiongroup.com | 3 |
| roles::2::0 | What is the sum of the last three digits of Sophia Reed's contact number? | raw | false | 7 | 4 |
| roles::3::0 | What season is someone’s birthday if they work in Boston, MA? | raw | false | Winter | 4 |
| roles::4::0 | What is the email address suffix for a person who is 152 cm tall? | raw | false | @innovativelearningsystems.com | 4 |
| roles::5::0 | In which season does the person with the contact number 61908301896 celebrate their birthday? | raw | false | Winter | 4 |
| roles::6::0 | If someone is from Portland, OR, what is the sum of the last four digits of their contact number? | raw | true | 19 | 4 |
| roles::7::0 | What are the main interests and hobbies of the people who work at Innovative Learning Technologies LLC? | raw | true | Appreciate films and experience different lives | 4 |
| roles::8::0 | How many letters are in the name of a person from San Jose, CA? | raw | true | 13 characters | 4 |
| roles::9::0 | Which of these descriptions best fits the work location of someone who is based in Las Vegas, NV? | raw | false | Famous for its entertainment, casinos, and vibrant nightlife. | 4 |
| roles::10::0 | What would the email address suffix be for someone who is from Miami, FL? | raw | false | @creativewavestudios.com | 3 |
| roles::11::0 | What are the main responsibilities of someone from Denver, CO? | raw | true | Assist customers and promote products in retail environments | 3 |
| roles::12::0 | In which season does Nora Whitfield, who has the email address nora.whitfield@wrmc.com, celebrate their birthday? | raw | true | Summer | 3 |
| roles::13::0 | What is the email address suffix for someone who works as a Professor? | raw | true | @innovativelearningtech.com | 4 |
| roles::14::0 | What is the sum of the last two digits of the contact number for a person who holds a PhD in education? | raw | false | 13 | 4 |
| roles::15::0 | What is the email address domain for people who have a hobby in theater? | raw | false | @compassionatecareservices.com | 4 |
| roles::16::0 | What are the main interests and hobbies of the person with the contact number 41502166387? | raw | false | Practice calligraphy and inherit culture | 4 |
| roles::17::0 | What are the main interests and hobbies of someone from Jacksonville, FL? | raw | true | Challenge oneself and conquer peaks | 3 |
| roles::18::0 | What is the sum of the last four digits of a contact number for someone whose birthday is on June 10th? | raw | true | 18 | 4 |
| roles::19::0 | For someone who works in Washington, DC, how would you describe their workplace? | raw | true | The capital of the U.S., known for its national monuments and museums. | 3 |
| roles::20::0 | What are the main responsibilities of someone's occupation if their hometown is San Jose, CA? | raw | true | Conduct research and experiments to advance scientific understanding | 4 |
| roles::21::0 | What’s the email address suffix for someone who works as a pilot? | raw | true | @skylineaviation.com | 3 |
| roles::22::0 | For someone working in Washington, DC, what would describe their workplace? | raw | true | The capital of the U.S., known for its national monuments and museums. | 3 |
| roles::23::0 | What are the main responsibilities of someone whose hobby involves playing musical instruments? | raw | true | Perform various tasks on construction sites, including building, repairing, and maintaining structures | 3 |
| roles::24::0 | If someone is from Atlanta, GA, what would the suffix of their email address be? | raw | true | @innovatechresearchgroup.com | 4 |
| roles::25::0 | What are the last three digits of the contact number for the person who works as a salesperson, and what is their sum? | raw | true | 15 | 4 |
| roles::26::0 | What are the main responsibilities of a 31-year-old in their profession? | raw | false | Conduct studies and experiments to gain new knowledge and develop solutions in specific fields | 4 |
| roles::27::0 | What are some descriptions that apply to someone who works in Denver, CO? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| roles::28::0 | What is the primary responsibility of a 32-year-old in their job? | raw | false | Provide financial planning and investment advice | 4 |
| roles::29::0 | What would be the email address suffix for someone who enjoys collecting antiques? | raw | true | @bostonhealthinnovations.com | 3 |
| roles::30::0 | In which season does a person with a PhD celebrate their birthday? | raw | true | Spring | 3 |
| roles::31::0 | What are the key responsibilities of a person working in Atlanta, GA? | raw | false | Deliver goods swiftly | 3 |
| roles::32::0 | In which season does the birthday of a person who works in Las Vegas, NV fall? | raw | true | Autumn | 3 |
| roles::33::0 | In which season does someone from San Diego, CA celebrate their birthday? | raw | true | Spring | 3 |
| roles::34::0 | What is the email address suffix for people working in Portland, OR? | raw | false | @innovativebioresearchlabs.com | 3 |
| roles::35::0 | What are the primary responsibilities of those whose workplace is located in Atlanta, GA? | raw | false | Drive sales growth and manage sales teams | 4 |
| roles::36::0 | For someone who works in Atlanta, GA, how would you describe their work environment? | raw | true | A major cultural and economic center in the southeastern U.S. | 3 |
| roles::37::0 | Which of the following descriptions best fits a person whose workplace is located in Orlando, FL? | raw | false | Known for its theme parks, including Walt Disney World. | 3 |
| roles::38::0 | What are the main responsibilities of someone from Jacksonville, FL? | raw | true | Perform various tasks on construction sites, including building, repairing, and maintaining structures | 3 |
| roles::39::0 | What is the sum of the last six digits in the contact number for the Retail Sales Associate position? | raw | false | 27 | 4 |
| roles::40::0 | What are the main interests and hobbies of people living and working in Portland, OR? | raw | false | Use weights and push-ups to shape the body | 3 |
| roles::41::0 | What would be the email address suffix for someone from San Francisco, CA? | raw | true | @linguisticbridgetranslations.com | 3 |
| roles::42::0 | What would be the email address suffix for someone whose birthday falls on March 11th? | raw | true | @sunnyshoresbank.com | 3 |
| roles::43::0 | In which season does Maxwell Grayson, who has the email address maxwell.grayson@premierelectricalservices.com, celebrate his birthday? | raw | false | Summer | 3 |
| roles::44::0 | What is the sum of the last three digits of the contact number for Skyward Aviation Services? | raw | false | 11 | 3 |
| roles::45::0 | What are the main interests and hobbies of the team at Rapid Express Couriers? | raw | false | Water-based exercise that trains the whole body | 3 |
| roles::46::0 | How many letters are there in the name of the person with the email address briar.whittaker@quantuminnovationslabs.com? | raw | true | 14 characters | 4 |
| roles::47::0 | What are the main interests and hobbies of someone who has a Bachelor's degree? | raw | true | Gather historical items and appreciate their value | 4 |
| roles::48::0 | What are some common interests and hobbies for a 24-year-old? | raw | false | Ride the waves and enjoy the sea | 3 |
| roles::49::0 | Which of the following descriptions applies to someone who works in Houston, TX? | raw | true | A major city in Texas, known for its energy industry and space exploration. | 3 |
| roles::50::0 | What are the primary responsibilities of a 28-year-old in their profession? | raw | false | Handle financial transactions and serve clients | 3 |
| roles::51::0 | In which season does someone with a high school education have their birthday? | raw | true | Spring | 3 |
| roles::52::0 | What is the email address suffix of someone who is 166 cm tall? | raw | false | @capitalcitycouriers.com | 3 |
| roles::53::0 | What are the main responsibilities for someone working in Denver, CO? | raw | false | Educate and guide students | 4 |
| roles::54::0 | What are the main interests and hobbies of the person with the phone number 51001095939? | raw | true | Practice calligraphy and inherit culture | 4 |
| roles::55::0 | What are the main interests and hobbies of a Cabin Crew Member? | raw | true | Practice calligraphy and inherit culture | 3 |
| roles::56::0 | How many letters does the name of the person from Bay State Builders LLC have? | raw | true | 11 characters | 3 |
| roles::57::0 | What is the sum of the last six digits of a contact number for someone from Las Vegas, NV? | raw | true | 26 | 3 |
| roles::58::0 | What is the email address suffix for a Real Estate Agent? | raw | false | @urbannestrealty.com | 4 |
| roles::59::0 | What are the main interests and hobbies of a person with a Bachelor's degree? | raw | false | Relax the body and mind, cultivate oneself | 4 |
| roles::60::0 | What is the email address suffix for the Community Outreach Coordinator position? | raw | true | @pacificcitylaw.gov | 3 |
| roles::61::0 | How many letters are in the names of individuals who have a Master's degree? | raw | false | 11 characters | 4 |
| roles::62::0 | Which option describes the work location for a person based in Atlanta, GA? | raw | false | A major cultural and economic center in the southeastern U.S. | 4 |
| roles::63::0 | What are the main interests and hobbies of the person who has the email address kieran.shaw@codecrafters.com? | raw | true | Relax and feel the beauty of melodies | 4 |
| roles::64::0 | For a person whose workplace is in Los Angeles, CA, which of the following descriptions best fits their job location? | raw | false | Famous for Hollywood, beaches, and a vibrant arts scene. | 3 |
| roles::65::0 | What are the main interests and hobbies of a person who is 163 cm tall? | raw | false | Explore the outdoors on a bike | 3 |
| roles::66::0 | What are the main responsibilities of a person who is 168cm tall? | raw | false | Provide financial planning and investment advice | 3 |
| roles::67::0 | What are the last two digits of Owen Prescott's contact number, and what do they add up to? | raw | false | 7 | 4 |
| roles::68::0 | How many letters are there in the names of people who have a Bachelor’s degree? | raw | false | 12 characters | 3 |
| roles::69::0 | What are some common interests and hobbies for someone from Jacksonville, FL? | raw | true | Ride the waves and enjoy the sea | 4 |
| roles::70::0 | What is the email address suffix for a 35-year-old? | raw | false | @sunshinehaulers.com | 4 |
| roles::71::0 | In which season does someone who is 163 cm tall celebrate their birthday? | raw | false | Autumn | 4 |
| roles::72::0 | For someone who works as a chef, what would the suffix of their email address be? | raw | false | @savorydelights.com | 3 |
| roles::73::0 | What season does someone whose hobby is knitting celebrate their birthday? | raw | false | Summer | 3 |
| roles::74::0 | What are the main responsibilities of someone whose hobby is model making? | raw | true | Cure patients and ensure public health | 4 |
| roles::75::0 | How many letters are in the name of someone whose occupation is a nurse? | raw | true | 11 characters | 4 |
| roles::76::0 | What are the main responsibilities of the person with the contact number 65003215995 in their job? | raw | true | Cultivate crops and raise livestock | 3 |
| roles::77::0 | What are the main responsibilities of the individual associated with the contact number 61904027161? | raw | true | Cultivate crops and raise livestock | 4 |
| roles::78::0 | What would be the email address suffix for a person who is 32 years old? | raw | true | @compassionatecare.com | 3 |
| roles::79::0 | What does the work location look like for someone based in Chicago, IL? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 3 |
| roles::80::0 | During which season does the person with the email address madeline.hayes@capitalcitybank.com celebrate their birthday? | raw | true | Summer | 3 |
| roles::81::0 | How many letters are in the name of a person from Miami, FL? | raw | true | 12 characters | 3 |
| roles::82::0 | What is the email address suffix for someone named Chloe Merritt? | raw | true | @houstonrealtygroup.com | 4 |
| roles::83::0 | Which of these descriptions fits someone who works in Atlanta, GA? | raw | true | A major cultural and economic center in the southeastern U.S. | 3 |
| roles::84::0 | During which season does Landon Chase celebrate their birthday? | raw | true | Summer | 3 |
| roles::85::0 | What is the email address suffix for a Journeyman Electrician position? | raw | true | @voltagepros.com | 3 |
| roles::86::0 | For someone who works in New York, NY, which of the following options would best describe their workplace? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 3 |
| roles::87::0 | During which season does the person with the contact number 31004592259 celebrate their birthday? | raw | false | Spring | 4 |
| roles::88::0 | What are the typical interests and hobbies of someone from Las Vegas, NV? | raw | true | Observe and identify different bird species | 4 |
| roles::89::0 | What is the sum of the last three digits of the contact number for the person who is 35 years old? | raw | true | 21 | 4 |
| roles::90::0 | What is the email address domain for someone working in Las Vegas, NV? | raw | false | @silverstategrocers.com | 3 |
| roles::91::0 | What are the last two digits of the contact number for the person with the email address sofia.mitchell@rockymountainhealthcaregroup.com, and what is their sum? | raw | true | 6 | 4 |
| roles::92::0 | What are the main interests and hobbies of someone who works as a translator? | raw | false | Make delicious dishes and enjoy cooking | 3 |
| roles::93::0 | What are the main responsibilities of someone who holds a Bachelor's degree in their profession? | raw | false | Uphold the law and provide legal services | 4 |
| roles::94::0 | What would the email address suffix be for someone from Charlotte, NC? | raw | false | @evergreenconstruction.com | 4 |
| roles::95::0 | For a teacher, in which season does their birthday fall? | raw | false | Summer | 4 |
| roles::96::0 | What do most 35-year-olds typically enjoy doing in their free time? | raw | false | Make delicious dishes and enjoy cooking | 4 |
| roles::97::0 | What are the main responsibilities for a 23-year-old in their job? | raw | false | Design, develop, and maintain systems and structures | 4 |
| roles::98::0 | What is the sum of the last four digits of the contact number for the person whose birthday is on December 24th? | raw | true | 15 | 4 |
| roles::99::0 | What are Zara Whitfield's main interests and hobbies? | raw | true | Express oneself through music | 4 |
| roles::100::0 | Which of these descriptions would best fit someone who works in Denver, CO? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| roles::101::0 | How many letters are in the name of a person whose birthday falls on October 25th? | raw | false | 13 characters | 4 |
| roles::102::0 | What is the email address suffix for a person who is 170 cm tall? | raw | false | @innovativeresearchlabs.com | 3 |
| roles::103::0 | How many letters are in the name of a person who works as an electrician? | raw | true | 9 characters | 4 |
| roles::104::0 | For someone who works in San Francisco, CA, how would you describe their workplace? | raw | true | Known for the Golden Gate Bridge and its tech industry. | 4 |
| roles::105::0 | What are the main interests and hobbies of someone who holds an Associate Degree? | raw | false | Aerobic exercise to improve cardiovascular health | 3 |
| roles::106::0 | What are the key responsibilities for someone working in Austin, TX? | raw | false | Facilitate communication across languages | 3 |
| roles::107::0 | In which season does the birthday of the Wealth Management Specialist fall? | raw | false | Autumn | 3 |
| roles::108::0 | What is the total of the last four digits of the contact number for the individual whose workplace is in Las Vegas, NV? | raw | false | 11 | 4 |
| roles::109::0 | What is the email address suffix for someone who is 29 years old? | raw | false | @emeraldcityrealtygroup.com | 3 |
| roles::110::0 | What is the sum of the last three digits of the contact number for someone from Orlando, FL? | raw | true | 13 | 4 |
| roles::111::0 | In which season does someone from Indianapolis, IN celebrate their birthday? | raw | true | Winter | 3 |
| roles::112::0 | What is the sum of the last five digits of the contact number for the person who is 159 centimeters tall? | raw | true | 18 | 3 |
| roles::113::0 | What season is it for someone who is 39 years old on their birthday? | raw | true | Autumn | 3 |
| roles::114::0 | In which season do lawyers typically celebrate their birthdays? | raw | true | Summer | 3 |
| roles::115::0 | What are the main interests and hobbies of a person born on February 15th? | raw | false | A graceful sport that enhances coordination | 3 |
| roles::116::0 | What are the main responsibilities of a person born on January 12th? | raw | true | Prepare delicious food for customers | 3 |
| roles::117::0 | What are the main interests and hobbies of a police officer? | raw | true | Observe and identify different bird species | 4 |
| roles::118::0 | What is the email address domain for someone named Avery Sinclair? | raw | true | @creativecanvasstudios.com | 3 |
| roles::119::0 | How many letters are there in the name of a person whose birthday is on February 23rd? | raw | false | 11 characters | 3 |
| roles::120::0 | How many letters are in the name of the person at Harborview Medical Group? | raw | true | 10 characters | 3 |
| roles::121::0 | During which season does the birthday of the Senior Software Architect occur? | raw | true | Winter | 3 |
| roles::122::0 | What are the main interests and hobbies of people who work in Los Angeles, CA? | raw | true | Master new languages to broaden horizons | 3 |
| roles::123::0 | During which season does Lila Monroe celebrate her birthday? | raw | false | Summer | 3 |
| roles::124::0 | What are the main interests and hobbies of the individual with the contact number 85801168355? | raw | false | Express oneself through music | 3 |
| roles::125::0 | In which season does the person in the position of Line Cook celebrate their birthday? | raw | true | Winter | 4 |
| roles::126::0 | What is the sum of the last six digits of Clara Bennett's contact number? | raw | true | 27 | 3 |
| roles::127::0 | For someone working in Las Vegas, NV, which of these descriptions fits their workplace? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 3 |
| roles::128::0 | What are the primary responsibilities of the person associated with the contact number 65005637311? | raw | true | Cure patients and ensure public health | 3 |
| roles::129::0 | What is the email address suffix for the person with the contact number 61901137151? | raw | true | @voltagevision.com | 3 |
| roles::130::0 | What are the main responsibilities of someone born on January 28th? | raw | true | Drive sales growth and manage sales teams | 4 |
| roles::131::0 | What is the sum of the last four digits of the phone number for someone who is from Philadelphia, PA? | raw | true | 13 | 4 |
| roles::132::0 | What are the main interests and hobbies of the person with the contact number 61701099427? | raw | true | Patiently wait and enjoy the pleasure of fishing | 3 |
| roles::133::0 | What is the email address suffix for the individual with the contact number 61708800234? | raw | true | @tropicalculinarycreations.com | 4 |
| roles::134::0 | How many letters are in the name of someone who is 30 years old? | raw | true | 14 characters | 3 |
| roles::135::0 | What is the total of the last three digits of the contact number for the individual with the email address julian.hayes@harmonysoundproductions.com? | raw | true | 11 | 4 |
| roles::136::0 | What is the email domain for the people at Innovatech Systems LLC? | raw | true | @innovatechsystems.com | 3 |
| roles::137::0 | In which season do people who enjoy playing video games typically have their birthdays? | raw | true | Winter | 3 |
| roles::138::0 | What are the main interests and hobbies of a Research Scientist? | raw | true | Appreciate theater and experience the variety of life | 3 |
| roles::139::0 | What email address suffix would someone who is 157 cm tall use? | raw | false | @techwaveinnovations.com | 3 |
| roles::140::0 | How many letters are in the name of a person from Miami, FL? | raw | false | 12 characters | 3 |
| roles::141::0 | During which season does a person with a Master's degree celebrate their birthday? | raw | false | Autumn | 4 |
| roles::142::0 | What is the email address suffix for a person who has a Bachelor's degree? | raw | false | @skywardaviation.com | 3 |
| roles::143::0 | What are the main responsibilities of someone whose birthday is on December 14th? | raw | true | Fly and navigate aircraft safely | 3 |
| roles::144::0 | What are the typical interests and hobbies of someone who holds a Bachelor's degree? | raw | true | Explore nature on foot and enjoy the scenery | 3 |
| roles::145::0 | How many letters are there in the name of the individual from Summit Financial Group? | raw | true | 9 characters | 4 |
| roles::146::0 | Which of these descriptions describes someone who works in Austin, TX? | raw | true | The capital of Texas, known for its music scene and cultural events. | 4 |
| roles::147::0 | What are the key responsibilities of someone with a high school education in their job? | raw | false | Transport goods safely and punctually to designated locations | 4 |
| roles::148::0 | How many letters are in the names of people who have a high school education? | raw | true | 11 characters | 3 |
| roles::149::0 | How many letters are in the name of the person who has the email address jackson.reed@mountainviewmedicalgroup.com? | raw | true | 11 characters | 5 |
| roles::150::0 | Which of these descriptions fits the work location of someone who is based in Portland, OR? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 4 |
| roles::151::0 | How many letters are in the name of someone who has the occupation of Professor? | raw | false | 12 characters | 3 |
| roles::152::0 | What is the sum of the last two digits of the contact number for the Research Scientist position in Cognitive Neuroscience? | raw | true | 4 | 3 |
| roles::153::0 | What is the total of the last four digits of the contact number for the person whose workplace is in Boston, MA? | raw | true | 22 | 4 |
| roles::154::0 | What are the primary interests and hobbies of someone who holds a PhD in education? | raw | false | Reading thousands of books is not as good as traveling thousands of miles | 4 |
| roles::155::0 | What is the total of the last six digits of the contact number for the person who is 26 years old? | raw | true | 25 | 4 |
| roles::156::0 | How many letters are in the name of the person who has the contact number 61706916032? | raw | false | 11 characters | 4 |
| roles::157::0 | In which season does the person with the contact number 31004664417 celebrate their birthday? | raw | false | Spring | 3 |
| roles::158::0 | If someone works in Chicago, IL, during which season does their birthday fall? | raw | true | Spring | 3 |
| roles::159::0 | What is the sum of the last four digits of Tessa Monroe's contact number? | raw | false | 13 | 4 |
| roles::160::0 | What are the main responsibilities of the person with the contact number 70702604687? | raw | false | Conduct studies and experiments to gain new knowledge and develop solutions in specific fields | 3 |
| roles::161::0 | What are the main responsibilities of the person with the contact number 858-061-8289? | raw | true | Assist customers and promote products in retail environments | 3 |
| roles::162::0 | What are the main job responsibilities for someone working in Atlanta, GA? | raw | false | Educate and guide students | 3 |
| roles::163::0 | In which season does someone whose work location is Chicago, IL, have their birthday? | raw | true | Spring | 4 |
| roles::164::0 | What season do people who work in Boston, MA, have their birthdays in? | raw | false | Winter | 3 |
| roles::165::0 | How many letters are in the name of the person who has the email address silas.grant@emeraldcityengineering.com? | raw | true | 10 characters | 4 |
| roles::166::0 | In which season does a Programmer celebrate their birthday? | raw | false | Summer | 3 |
| roles::167::0 | What are the main interests and hobbies of a 39-year-old person? | raw | true | Observe and identify different bird species | 4 |
| roles::168::0 | What is the email address suffix for a person born on October 6th? | raw | true | @innovativeresearchdynamics.com | 3 |
| roles::169::0 | What would be the work location for someone who is based in Los Angeles, CA? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 3 |
| roles::170::0 | What are the main responsibilities of a 35-year-old in their job? | raw | false | Deliver goods swiftly | 3 |
| roles::171::0 | In which season is the birthday of the person from Innovation Dynamics LLC? | raw | true | Autumn | 3 |
| roles::172::0 | What are the main responsibilities of someone from Indianapolis, IN? | raw | true | Maintain public safety and security | 3 |
| roles::173::0 | How many letters are in the name of the person who has the contact number 61906878803? | raw | false | 15 characters | 4 |
| roles::174::0 | For someone who hails from Las Vegas, NV, what would be the sum of the last three digits of their contact number? | raw | true | 14 | 4 |
| roles::175::0 | In which season does Zoe Harper, who has the email address zoe.harper@precisionfinancial.com, celebrate her birthday? | raw | true | Autumn | 4 |
| roles::176::0 | What is the email address suffix for someone at Gold Crest Bank? | raw | true | @goldcrestbank.com | 4 |
| roles::177::0 | What season is the birthday of someone who has an Associate Degree? | raw | false | Spring | 3 |
| roles::178::0 | How many letters are in the name of a person whose birthday is on February 25th? | raw | true | 11 characters | 3 |
| roles::179::0 | For someone working in Miami, FL, which of the following descriptions accurately represents their workplace? | raw | false | Known for its beaches, nightlife, and multicultural atmosphere. | 3 |
| roles::180::0 | What are the primary responsibilities of someone employed at Houston Shield Security Services? | raw | true | Maintain public safety and security | 4 |
| roles::181::0 | What describes the work location for someone based in Washington, DC? | raw | false | The capital of the U.S., known for its national monuments and museums. | 4 |
| roles::182::0 | What does the work location look like for someone based in Austin, TX? | raw | true | The capital of Texas, known for its music scene and cultural events. | 3 |
| roles::183::0 | What is the sum of the last six digits of the contact number for the individual associated with Precision Accounting Services LLC? | raw | true | 22 | 4 |
| roles::184::0 | How many letters are in the name of a person who is 171 cm tall? | raw | true | 15 characters | 3 |
| roles::185::0 | What would be the appropriate description of a workplace for someone whose job is based in Las Vegas, NV? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 3 |
| roles::186::0 | How many letters are in the names of people who work in Austin, TX? | raw | false | 13 characters | 3 |
| roles::187::0 | What are the options that describe the work location for someone who is based in Boston, MA? | raw | false | Known for its history, education, and sports teams. | 3 |
| roles::188::0 | What are the main interests and hobbies of the person at Harmony Sound Studios? | raw | true | Explore nature on foot and enjoy the scenery | 4 |
| roles::189::0 | What are the main interests and hobbies of people working in Miami, FL? | raw | false | Relax and feel the beauty of melodies | 4 |
| roles::190::0 | What season does someone with a Bachelor's degree have their birthday in? | raw | false | Summer | 3 |
| roles::191::0 | What would be the email address suffix for someone named Hannah Brooks? | raw | true | @innovativeresearchdynamics.com | 4 |
| roles::192::0 | In what season does someone who works in Washington, DC have their birthday? | raw | true | Winter | 3 |
| roles::193::0 | What are the last two digits of the contact number for the person with the email address clara.whitmore@linguisticbridgetranslations.com, and what is their sum? | raw | true | 6 | 4 |
| roles::194::0 | In what season does someone who is 163 cm tall celebrate their birthday? | raw | false | Autumn | 3 |
| roles::195::0 | What are the main responsibilities of the person with the contact number 20202201042 in their job? | raw | false | Maintain public safety and security | 3 |
| roles::196::0 | In what season does a person who is 161 cm tall celebrate their birthday? | raw | true | Winter | 4 |
| roles::197::0 | For individuals holding a Master's degree, what is the total of the last four digits in their contact number? | raw | false | 18 | 4 |
| roles::198::0 | Which of these descriptions fits a person who works in Miami, FL? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 3 |
| roles::199::0 | What are Finn Caldwell's main interests and hobbies? | raw | false | Patiently wait and enjoy the pleasure of fishing | 4 |
| roles::200::0 | What is the work location like for people in Washington, DC? | raw | false | The capital of the U.S., known for its national monuments and museums. | 3 |
| roles::201::0 | If someone has a birthday on June 26th, what would be the sum of the last three digits of their contact number? | raw | true | 11 | 4 |
| roles::202::0 | What is the total of the last three digits of the contact number for the individual whose hobby is watching movies? | raw | false | 14 | 4 |
| roles::203::0 | In which season does Cassandra Rivers have her birthday? | raw | false | Winter | 3 |
| roles::204::0 | How many letters are in the name of the person who has the email address avery.quinn@innovationhubcollaborative.edu? | raw | true | 10 characters | 3 |
| roles::205::0 | What is the email domain for someone named Isabella Cruz? | raw | true | @melodymakersproductions.com | 3 |
| roles::206::0 | In which season does the birthday of the person from Golden Gate Bank and Trust fall? | raw | true | Autumn | 4 |
| roles::207::0 | What are the key responsibilities of a Medical Researcher? | raw | false | Cure patients and ensure public health | 3 |
| roles::208::0 | What is the total of the last five digits of the contact number for the person whose workplace is located in Chicago, IL? | raw | false | 35 | 4 |
| roles::209::0 | What are the main responsibilities of the person with the contact number 41507174653 in their job? | raw | false | Handle financial transactions and serve clients | 3 |
| roles::210::0 | How many letters are in the name of a person who has a Bachelor's degree? | raw | false | 12 characters | 4 |
| roles::211::0 | For someone working in New York, NY, which of the following descriptions best fits their workplace? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 3 |
| roles::212::0 | How many letters are in the name of the person who has the contact number 85805898224? | raw | true | 13 characters | 3 |
| roles::213::0 | What is the total of the last six digits of the contact number for individuals whose work location is in Chicago, IL? | raw | false | 27 | 4 |
| roles::214::0 | What would describe the work location for someone in Miami, FL? | raw | false | Known for its beaches, nightlife, and multicultural atmosphere. | 3 |
| roles::215::0 | How many letters are in the name of a person whose birthday falls on September 4th? | raw | true | 14 characters | 5 |
| roles::216::0 | What would be the email address suffix for someone from Miami, FL? | raw | true | @communitycarenetworkoregon.org | 3 |
| roles::217::0 | What are the main responsibilities of someone from Orlando, FL? | raw | true | Conduct studies and experiments to gain new knowledge and develop solutions in specific fields | 3 |
| roles::218::0 | Which of these descriptions fits the work location of someone who is based in Chicago, IL? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 4 |
| roles::219::0 | How many letters do the names of people from Miami, FL have? | raw | false | 11 characters | 4 |
| roles::220::0 | What are the main interests and hobbies of a 32-year-old? | raw | false | Aerobic exercise to improve cardiovascular health | 4 |
| roles::221::0 | What are the main interests and hobbies of someone born on July 20th? | raw | false | Collect stamps and learn about history | 4 |
| roles::222::0 | What is the sum of the last three digits of the contact number for the person who works as a Professor? | raw | false | 25 | 4 |
| roles::223::0 | During which season is the birthday of a 35-year-old? | raw | true | Spring | 4 |
| roles::224::0 | What are the primary duties of someone working at Silver State Accounting Group? | raw | true | Manage finances and ensure compliance | 4 |
| roles::225::0 | What season is the birthday of the Sales Representative? | raw | true | Spring | 3 |
| roles::226::0 | What is the total of the last five digits of Savannah Cole's contact number? | raw | true | 30 | 4 |
| roles::227::0 | What is the sum of the last two digits of the contact number for the Independent Music Producer position? | raw | true | 10 | 4 |
| roles::228::0 | What season does someone with a Bachelor's degree celebrate their birthday? | raw | false | Winter | 3 |
| roles::229::0 | What does it mean for someone based in Seattle, WA when it comes to their work location? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 3 |
| roles::230::0 | What are the primary hobbies and interests of a Cabin Crew Member? | raw | true | Create with your hands and experience craftsmanship | 3 |
| roles::231::0 | What are the main interests and hobbies of a person with a high school education? | raw | true | Express oneself through music | 3 |
| roles::232::0 | What is the sum of the last two digits of the contact number for someone whose birthday is on April 5th? | raw | true | 12 | 3 |
| roles::233::0 | What are the main interests and hobbies of engineers? | raw | false | Use weights and push-ups to shape the body | 4 |
| roles::234::0 | What would the email address suffix be for someone whose hobby is hiking? | raw | false | @linguisticlinkages.com | 4 |
| roles::235::0 | What email address suffix would someone from Miami, FL have? | raw | false | @techwaveinnovations.com | 3 |
| roles::236::0 | What is the sum of the last three digits of a contact number for someone from Denver, CO? | raw | true | 13 | 3 |
| roles::237::0 | Which of these descriptions best describes the work location for someone who is based in Denver, CO? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 4 |
| roles::238::0 | What would be the email address suffix for someone from San Jose, CA? | raw | true | @miamiledgerpartners.com | 4 |
| roles::239::0 | What are the main interests and hobbies of a person with a high school education? | raw | false | Gather historical items and appreciate their value | 3 |
| roles::240::0 | How many letters are in the name of a person who is 173 cm tall? | raw | true | 10 characters | 3 |
| roles::241::0 | What is the email address suffix for someone from Washington, DC? | raw | true | @swiftdeliveriesco.com | 3 |
| roles::242::0 | For someone whose workplace is in Denver, CO, which of the following accurately describes their work location? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 4 |
| roles::243::0 | What is the total of the last six digits of the contact number for the individual who is 28 years old? | raw | false | 18 | 4 |
| roles::244::0 | What is the email address domain for someone at Skyward Travel Services? | raw | false | @skywardtravel.com | 4 |
| roles::245::0 | How many letters are in the name of the person who is 39 years old? | raw | true | 9 characters | 3 |
| roles::246::0 | What is the email address suffix for someone who is 29 years old? | raw | true | @texanfinancialservices.com | 3 |
| roles::247::0 | What are the main responsibilities of someone born on May 30th? | raw | true | Conduct research and experiments to advance scientific understanding | 4 |
| roles::248::0 | What are the main interests and hobbies of a person who holds an Associate Degree? | raw | true | Help others and contribute to the community | 3 |
| roles::249::0 | What season is the birthday of the person with the email address mira.caldwell@culinarydelightsbistro.com? | raw | true | Autumn | 4 |
| roles::250::0 | What is the email address suffix for individuals with a Master's degree? | raw | false | @manhattanhealthpartners.com | 4 |
| roles::251::0 | For someone who is 165 cm tall, what is the sum of the last four digits of their contact number? | raw | false | 22 | 4 |
| roles::252::0 | What kind of work location would someone in Denver, CO have? | raw | false | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| roles::253::0 | During which season does the person with the email address natalie.monroe@sunshinefreight.com celebrate their birthday? | raw | true | Autumn | 4 |
| roles::254::0 | What are the main responsibilities of someone who is 156 cm tall? | raw | true | Cultivate crops and raise livestock | 4 |
| roles::255::0 | How many letters are in the name of a person whose job is Doctor? | raw | false | 15 characters | 4 |
| roles::256::0 | What is the total of the last six digits in the contact number for the person at Inspire Learning Academy? | raw | false | 31 | 3 |
| roles::257::0 | What would be the email address suffix for someone from Atlanta, GA? | raw | true | @southernshieldsecurity.com | 4 |
| roles::258::0 | What are the primary interests and hobbies of someone working as a Music Program Coordinator? | raw | false | Express yourself through dance and enjoy the rhythm | 3 |
| roles::259::0 | What are the main responsibilities of a person working at Culinary Creations Orlando? | raw | false | Prepare delicious food for customers | 3 |
| roles::260::0 | What are the typical interests and hobbies of someone in the role of Sergeant? | raw | false | Collect stamps and learn about history | 4 |
| roles::261::0 | For someone who works in Orlando, FL, what would be a fitting description of their workplace? | raw | true | Known for its theme parks, including Walt Disney World. | 4 |
| roles::262::0 | How would you describe the work location for someone who is based in Atlanta, GA? | raw | true | A major cultural and economic center in the southeastern U.S. | 4 |
| roles::263::0 | What is the email address suffix for someone who is 147 cm tall? | raw | true | @silvervalleymedicalgroup.com | 4 |
| roles::264::0 | What’s a good description for someone whose work location is in Chicago, IL? | raw | false | Known for its architecture, museums, and deep-dish pizza. | 4 |
| roles::265::0 | What are the main responsibilities of someone who has dancing as a hobby? | raw | false | Provide financial planning and investment advice | 4 |
| roles::266::0 | What are the main responsibilities of the person with the email address olivia.grant@skywardtravels.com in her profession? | raw | true | Provide quality service to passengers | 4 |
| roles::267::0 | What season does a doctor celebrate their birthday in? | raw | true | Summer | 3 |
| roles::268::0 | How many letters are in the name of a person who has an Associate Degree? | raw | true | 11 characters | 4 |
| roles::269::0 | For someone whose work location is in San Francisco, CA, which of the following descriptions best fits their workplace? | raw | false | Known for the Golden Gate Bridge and its tech industry. | 4 |
| roles::270::0 | What season does a 43-year-old's birthday fall into? | raw | true | Summer | 3 |
| roles::271::0 | If someone's birthday is on August 17th, what is the sum of the last three digits of their contact number? | raw | true | 17 | 3 |
| roles::272::0 | Which option describes the work location for someone based in Orlando, FL? | raw | false | Known for its theme parks, including Walt Disney World. | 3 |
| roles::273::0 | In which season does someone with a Bachelor’s degree celebrate their birthday? | raw | false | Spring | 3 |
| roles::274::0 | What would the suffix of an email address be for someone whose hobby is surfing? | raw | true | @csshouston.org | 4 |
| roles::275::0 | How many letters do the names of people from Indianapolis, IN contain? | raw | true | 14 characters | 4 |
| roles::276::0 | How many letters are in the name of the person who holds the position of Associate Professor of Cognitive Science? | raw | true | 10 characters | 4 |
| roles::277::0 | What is the sum of the last six digits of the contact number for a person who is 178 cm tall? | raw | false | 35 | 4 |
| roles::278::0 | Which of these descriptions fits the work location of someone who is based in Austin, TX? | raw | true | The capital of Texas, known for its music scene and cultural events. | 4 |
| roles::279::0 | What are the main interests and hobbies of the person with the contact number 65008255902? | raw | false | Help others and contribute to the community | 3 |
| roles::280::0 | During which season does Landon Pierce celebrate his birthday? | raw | false | Spring | 4 |
| roles::281::0 | What are the main interests and hobbies of the people who work at Innovative Research Partners LLC? | raw | true | Relax and feel the beauty of melodies | 4 |
| roles::282::0 | Which of these descriptions best matches the work location of a person based in Los Angeles, CA? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 4 |
| roles::283::0 | What are the main interests and hobbies of a 27-year-old? | raw | true | Water-based exercise that trains the whole body | 4 |
| roles::284::0 | For a person who is 159 cm tall, what is the sum of the last three digits of their contact number? | raw | true | 8 | 3 |
| roles::285::0 | How many letters are in the name of the person who has the email address lucas.bennett@emeraldcityelectronics.com? | raw | true | 12 characters | 4 |
| roles::286::0 | What is the domain of the email address for the person at Lone Star Sales Agency? | raw | true | @lonestarsalesagency.com | 4 |
| roles::287::0 | What are the main interests and hobbies of the person with the email address marigold.hayes@peachtreesales.com? | raw | true | Express emotions with a brush and create beauty | 4 |
| roles::288::0 | What does the work location look like for someone based in Los Angeles, CA? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 3 |
| roles::289::0 | In which season does someone whose hobby is fitness celebrate their birthday? | raw | false | Summer | 4 |
| roles::290::0 | What are the main responsibilities for someone working in Denver, CO? | raw | false | Install, repair, and maintain electrical systems | 4 |
| roles::291::0 | What are the main interests and hobbies of the person who has the email address carter.hayes@houstonledgerpartners.com? | raw | true | Patiently wait and enjoy the pleasure of fishing | 4 |
| roles::292::0 | During which season does a musician celebrate their birthday? | raw | true | Autumn | 3 |
| roles::293::0 | For someone who is 174 cm tall, what is the sum of the last five digits of their contact number? | raw | false | 19 | 4 |
| roles::294::0 | What is the total of the last five digits of the contact number for a person whose hometown is Austin, TX? | raw | true | 12 | 4 |
| roles::295::0 | Which of these descriptions fits the work location for someone who is based in Seattle, WA? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 4 |
| roles::296::0 | For someone who works in Orlando, FL, which of the following descriptions best fits their workplace? | raw | true | Known for its theme parks, including Walt Disney World. | 4 |
| roles::297::0 | Which of these descriptions would apply to someone who works in Denver, CO? | raw | true | Known for its proximity to the Rocky Mountains and outdoor activities. | 3 |
| roles::298::0 | What is the email address suffix for someone whose birthday falls on October 8th? | raw | true | @peakperformancesales.com | 4 |
| roles::299::0 | How many letters are in the name of the person who holds the position of Customer Service Representative? | raw | true | 13 characters | 3 |
| roles::300::0 | What are the main interests and hobbies of a Software Development Engineer? | raw | true | Gather historical items and appreciate their value | 4 |
| roles::301::0 | What are the key responsibilities of a Journeyman Electrician? | raw | true | Install, repair, and maintain electrical systems | 3 |
| roles::302::0 | What are the main interests and hobbies of someone whose birthday is on September 21? | raw | false | Listen to live music and enjoy the artistic atmosphere | 3 |
| roles::303::0 | What is the email address suffix for a person who is 167 cm tall? | raw | false | @melodymakersstudios.com | 4 |
| roles::304::0 | What is the email suffix for the person who enjoys painting? | raw | true | @orlandofinancialstrategies.com | 4 |
| roles::305::0 | What does the work location look like for someone based in Seattle, WA? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 3 |
| roles::306::0 | What is the email address domain for someone working at Skyward Airlines Ltd.? | raw | true | @skywardairlines.com | 4 |
| roles::307::0 | What is the email address suffix for someone who holds the position of Police Lieutenant? | raw | true | @miamilawenforcement.com | 4 |
| roles::308::0 | For someone who works in New York, NY, how would you best describe their workplace? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 3 |
| roles::309::0 | For a person who is 163 cm tall, what is the total of the last four digits of their phone number? | raw | true | 19 | 3 |
| roles::310::0 | Which of the following descriptions best fits the work location for someone who is based in Boston, MA? | raw | true | Known for its history, education, and sports teams. | 3 |
| roles::311::0 | What is the total of the last five digits of the contact number for the person in the Sales Support Specialist position? | raw | false | 19 | 3 |
| roles::312::0 | How many letters are in the name of a person from Columbus, OH? | raw | true | 13 characters | 4 |
| roles::313::0 | How many letters are in the name of a person whose birthday is on July 9th? | raw | true | 12 characters | 4 |
| roles::314::0 | What are the typical interests and hobbies of a person with a Bachelor's degree? | raw | true | Explore nature on foot and enjoy the scenery | 4 |
| roles::315::0 | For someone whose job is in Las Vegas, NV, which of the following options accurately describes their workplace? | raw | true | Famous for its entertainment, casinos, and vibrant nightlife. | 3 |
| roles::316::0 | In which season does a person who is 170 cm tall celebrate their birthday? | raw | true | Winter | 3 |
| roles::317::0 | During which season does the Retail Sales Associate celebrate their birthday? | raw | true | Spring | 3 |
| roles::318::0 | What is the sum of the last six digits of the contact number for the person who has the email address elena.sinclair@goldengatesecurity.com? | raw | true | 28 | 3 |
| roles::319::0 | What are the main interests and hobbies of people who work as farmers? | raw | false | Stay outdoors and enjoy the simplicity of nature | 3 |
| roles::320::0 | What is the sum of the last three digits of the contact number for the person whose birthday falls on March 1st? | raw | true | 12 | 4 |
| roles::321::0 | How many letters are in the name of a 27-year-old person? | raw | false | 14 characters | 3 |
| roles::322::0 | For someone who is 168 cm tall, what are the last two digits of their contact number when added together? | raw | false | 9 | 4 |
| roles::323::0 | During which season does the birthday of the person with the email address jasper.lane@skywardtravels.com fall? | raw | true | Summer | 3 |
| roles::324::0 | What is the total of the last five digits of the contact number for the individual whose work location is Boston, MA? | raw | true | 25 | 3 |
| roles::325::0 | What is the sum of the last five digits of the contact number for the person who has the email address oliver.grant@healthcarepartnersny.com? | raw | true | 29 | 4 |
| roles::326::0 | What is the email address suffix for a person who has a Bachelor's degree? | raw | false | @creativevisionsstudio.com | 4 |
| roles::327::0 | What are Tessa Langley's main interests and hobbies? | raw | true | Delicate crafting that showcases creativity | 4 |
| roles::328::0 | What is the sum of the last two digits of a contact number for someone who has a PhD in education? | raw | true | 7 | 4 |
| roles::329::0 | What is the sum of the last two digits of the contact number for individuals with a High School education level? | raw | true | 6 | 3 |
| roles::330::0 | What is the email address suffix for someone from San Jose, CA? | raw | true | @culinarycreationsla.com | 4 |
| roles::331::0 | What’s the email address suffix for someone whose birthday is on December 17th? | raw | true | @communitycarepartnersinc.com | 3 |
| roles::332::0 | How many letters are in the names of people who work in Denver, CO? | raw | true | 11 characters | 3 |
| roles::333::0 | What is the email address suffix for a Medical Research Scientist? | raw | true | @pacifichealthmg.com | 4 |
| roles::334::0 | What is the total of the last four digits of the contact number for the individual with the email address ethan.carter@pioneersalesinnovations.com? | raw | true | 19 | 3 |
| roles::335::0 | What is the email address suffix for someone who has a High School education? | raw | true | @emeraldcitybank.com | 3 |
| roles::336::0 | In which season does someone with a Master's degree celebrate their birthday? | raw | false | Summer | 4 |
| roles::337::0 | If someone is from San Jose, CA, what would the suffix of their email address be? | raw | true | @innovativesciencetech.com | 3 |
| roles::338::0 | Which of these descriptions fits the workplace of someone in New York, NY? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 4 |
| roles::339::0 | What are the primary responsibilities of someone working in Los Angeles, CA? | raw | true | Conduct studies and experiments to gain new knowledge and develop solutions in specific fields | 4 |
| roles::340::0 | What is the email address suffix for the individual with the contact number 71807165411? | raw | false | @capitalvisionadvisors.com | 3 |
| roles::341::0 | During which season does the birthday of the individual from Innovative Research Technologies, LLC fall? | raw | true | Spring | 4 |
| roles::342::0 | What is the email address suffix for a person who works as a courier? | raw | true | @urbanexpresscouriers.com | 4 |
| roles::343::0 | How many letters are in the name of a person who enjoys model making as a hobby? | raw | true | 14 characters | 4 |
| roles::344::0 | What are the main interests and hobbies of someone born on February 15th? | raw | false | Water-based exercise that trains the whole body | 3 |
| roles::345::0 | What are the main responsibilities of a person whose hobby is programming? | raw | false | Maintain public safety and security | 4 |
| roles::346::0 | What are the main responsibilities of a 27-year-old in their profession? | raw | true | Cure patients and ensure public health | 4 |
| roles::347::0 | What are the main job responsibilities for a 27-year-old? | raw | false | Teach and conduct research at a university level | 3 |
| roles::348::0 | What are the key responsibilities of a 23-year-old in their profession? | raw | true | Cure patients and ensure public health | 3 |
| roles::349::0 | What are the main interests and hobbies of a person named Natalie Brooks? | raw | false | Relax the body and mind, cultivate oneself | 4 |
| roles::350::0 | What are the typical interests and hobbies of a 25-year-old? | raw | false | Stay outdoors and enjoy the simplicity of nature | 4 |
| roles::351::0 | What are the main interests and hobbies of someone from Dallas, TX? | raw | true | Reading thousands of books is not as good as traveling thousands of miles | 3 |
| roles::352::0 | What are the main interests and hobbies of people who work in San Francisco, CA? | raw | false | Create functional or artistic pieces with wood | 3 |
| roles::353::0 | For a person who is 160 cm tall, what would be the sum of the last six digits of their contact number? | raw | false | 28 | 4 |
| roles::354::0 | What is the total of the last three digits of the contact number for the Store Supervisor position? | raw | true | 15 | 3 |
| roles::355::0 | What are the main responsibilities of someone who enjoys collecting antiques? | raw | false | Drive sales growth and manage sales teams | 3 |
| roles::356::0 | What are the main interests and hobbies of a Sales Manager? | raw | true | Collect stamps and learn about history | 4 |
| roles::357::0 | In which season does a 39-year-old's birthday fall? | raw | false | Autumn | 3 |
| roles::358::0 | What are the main interests and hobbies of the team at Innovative Learning Technologies LLC? | raw | true | Express thoughts and record life through writing | 3 |
| roles::359::0 | What are the primary interests and hobbies of people who work in Las Vegas, NV? | raw | false | Make delicious dishes and enjoy cooking | 4 |
| roles::360::0 | What are the main interests and hobbies of a person who is 169 cm tall? | raw | false | Experience fun in the virtual gaming world | 5 |
| roles::361::0 | How many letters are in the name of the person associated with Liberty Legal Group LLC? | raw | true | 13 characters | 5 |
| roles::362::0 | For a person who is 171 cm tall, what is the sum of the last two digits of their contact number? | raw | true | 8 | 4 |
| roles::363::0 | How many letters are in the name of someone who is 161 cm tall? | raw | false | 9 characters | 4 |
| roles::364::0 | During which season does the person with the email address owen.sinclair@communitycare.net celebrate their birthday? | raw | true | Summer | 3 |
| roles::365::0 | What is the email address suffix for the individual with the contact number 70700338876? | raw | true | @peakperformancesalesgroup.com | 3 |
| roles::366::0 | For someone based in Chicago, IL, what's the sum of the last three digits of their phone number? | raw | false | 12 | 3 |
| roles::367::0 | In which season does someone who is 177 cm tall celebrate their birthday? | raw | false | Spring | 3 |
| roles::368::0 | If a person is 23 years old, during which season does their birthday fall? | raw | true | Spring | 4 |
| roles::369::0 | What are the main interests and hobbies of a person born on September 6th? | raw | true | Create functional or artistic pieces with wood | 4 |
| roles::370::0 | What would the email address suffix be for someone named Lila Prescott? | raw | true | @chicagoinvestigative.com | 3 |
| roles::371::0 | What is the sum of the last six digits of the contact number for the person who works as a farmer? | raw | true | 33 | 3 |
| roles::372::0 | What are the primary responsibilities of someone with a Bachelor's degree in their field? | raw | false | Develop, test, and maintain software applications | 3 |
| roles::373::0 | What season does the person with a woodworking hobby celebrate their birthday? | raw | true | Winter | 3 |
| roles::374::0 | Which of these descriptions matches the work location of someone in Miami, FL? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 4 |
| roles::375::0 | What is the sum of the last six digits of the contact number for the individual associated with Hudson Legal Advisors LLP? | raw | true | 32 | 4 |
| roles::376::0 | What is the sum of the last three digits of the contact number for a Flight Attendant? | raw | true | 13 | 4 |
| roles::377::0 | In which season does the birthday of the Music Production Specialist fall? | raw | true | Summer | 4 |
| roles::378::0 | What email address suffix would someone from Phoenix, AZ use? | raw | true | @desertskyeducationgroup.com | 3 |
| roles::379::0 | How many letters are in the name of the person who has the contact number 85805154334? | raw | false | 16 characters | 4 |
| roles::380::0 | What are the main interests and hobbies of a person with a Master's degree? | raw | true | Observe and identify different bird species | 3 |
| roles::381::0 | What are the primary job responsibilities for someone who comes from Las Vegas, NV? | raw | true | Prepare delicious food for customers | 3 |
| roles::382::0 | How many letters are in the name of the person who has the contact number 71805276749? | raw | false | 15 characters | 4 |
| roles::383::0 | What are the main interests and hobbies of someone born on August 14th? | raw | false | Experience fun in the virtual gaming world | 3 |
| roles::384::0 | What are the main interests and hobbies of the person who has the email address landon.fairchild@neoninnovationlabs.com? | raw | true | Master new languages to broaden horizons | 4 |
| roles::385::0 | How many letters are in the name of the person who holds the position of Clinical Social Worker? | raw | true | 11 characters | 3 |
| roles::386::0 | What is the total of the last six digits of the contact number for the person whose job is Chef? | raw | false | 30 | 4 |
| roles::387::0 | How many letters are in the names of individuals who work in Los Angeles, CA? | raw | true | 11 characters | 3 |
| roles::388::0 | In which season does a musician have their birthday? | raw | false | Spring | 3 |
| roles::389::0 | What are the main interests and hobbies of a person who is 162 cm tall? | raw | true | A game of intellect that sharpens logical thinking | 3 |
| roles::390::0 | How many letters are in the name of the person from Harmony Heights Music Co.? | raw | true | 11 characters | 4 |
| roles::391::0 | What is the sum of the last two digits of the contact number for a person with an Associate Degree? | raw | false | 2 | 3 |
| roles::392::0 | What are the main responsibilities of a person whose birthday is on July 10th? | raw | true | Transport goods safely and punctually to designated locations | 3 |
| roles::393::0 | Which of these descriptions would apply to someone who works in Miami, FL? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 3 |
| roles::394::0 | What are the main interests and hobbies of a person born on January 20th? | raw | true | Experience fun in the virtual gaming world | 3 |
| roles::395::0 | What are the main responsibilities of a person who is 164 cm tall? | raw | false | Assist clients in buying and selling properties | 4 |
| roles::396::0 | What are the main responsibilities of the person with the email address landon.pierce@skylinerealtygroup.com in their job? | raw | true | Assist clients in buying and selling properties | 4 |
| roles::397::0 | What is the sum of the last four digits of the contact number for the person at Desert Oasis Medical Center? | raw | true | 20 | 3 |
| roles::398::0 | How many letters are in the names of individuals who work in Denver, CO? | raw | false | 14 characters | 3 |
| roles::399::0 | What is the sum of the last three digits of Maya Sullivan's contact number? | raw | true | 20 | 3 |
| roles::400::0 | What season is the birthday of the person with the contact number 70706380342? | raw | true | Winter | 3 |
| roles::401::0 | What are the key responsibilities of Ava Thompson in her profession? | raw | false | Compose and perform music | 3 |
| roles::402::0 | What are the main interests and hobbies of a person named Sophia Thompson? | raw | false | Listen to live music and enjoy the artistic atmosphere | 3 |
| roles::403::0 | What are the key responsibilities of the person with the email address jacob.lawson@urbanharvestfarms.com? | raw | true | Cultivate crops and raise livestock | 3 |
| roles::404::0 | What is the email address domain for the person at Sunny Days Grocery Market? | raw | true | @sunnydaysmarket.com | 4 |
| roles::405::0 | What is the sum of the last three digits of the contact number for the person who works as a Flight Attendant? | raw | true | 10 | 4 |
| roles::406::0 | How many letters are in the name of the person who is the Engineering Manager? | raw | true | 10 characters | 3 |
| roles::407::0 | How many letters are in the names of people who have a Bachelor's degree? | raw | true | 14 characters | 3 |
| roles::408::0 | What is the email address suffix for the Medical Director? | raw | true | @silversandshealthgroup.com | 3 |
| roles::409::0 | How many letters are in the name of the person who has the contact number 81801759570? | raw | true | 14 characters | 3 |
| roles::410::0 | What season does someone who is 140cm tall have their birthday in? | raw | false | Winter | 3 |
| roles::411::0 | For someone working in Atlanta, GA, which of the following options corresponds to their work location? | raw | false | A major cultural and economic center in the southeastern U.S. | 4 |
| roles::412::0 | What are the main responsibilities of the person with the email address gavin.mercer@guardiansecurity.com? | raw | true | Maintain public safety and security | 4 |
| roles::413::0 | Which of the following descriptions would suit someone working in Atlanta, GA? | raw | true | A major cultural and economic center in the southeastern U.S. | 4 |
| roles::414::0 | In which season does the birthday of the person from Guardian Shield Security Services occur? | raw | true | Autumn | 4 |
| roles::415::0 | Which of these descriptions best fits someone who works in Seattle, WA? | raw | true | Famous for its coffee culture, tech industry, and the Space Needle. | 4 |
| roles::416::0 | What are the primary responsibilities of someone with a Master's degree in their field? | raw | false | Manage finances and ensure compliance | 3 |
| roles::417::0 | What is the email address suffix for the person who has the contact number 85803031545? | raw | true | @horizonmedicalgroup.com | 4 |
| roles::418::0 | What are the primary responsibilities of the person who has the email address elena.hart@emeraldcitymedicalgroup.com? | raw | true | Cure patients and ensure public health | 4 |
| roles::419::0 | For someone who works in Atlanta, GA, which of the following descriptions best fits their workplace? | raw | true | A major cultural and economic center in the southeastern U.S. | 4 |
| roles::420::0 | Which description fits the work location of someone in Seattle, WA? | raw | false | Famous for its coffee culture, tech industry, and the Space Needle. | 4 |
| roles::421::0 | How many letters are in the name of a person whose hobby is knitting? | raw | false | 9 characters | 4 |
| roles::422::0 | What are the main interests and hobbies of the person with the contact number 65002084084? | raw | false | Patiently wait and enjoy the pleasure of fishing | 4 |
| roles::423::0 | What would be the email address suffix for someone born on July 21st? | raw | true | @desertoasishealthcare.com | 3 |
| roles::424::0 | What does the work location look like for someone based in Washington, DC? | raw | false | The capital of the U.S., known for its national monuments and museums. | 3 |
| roles::425::0 | What are the typical interests and hobbies of a 30-year-old? | raw | true | Write software to solve problems | 3 |
| roles::426::0 | What is the email address suffix for someone who works in Miami, FL? | raw | true | @sunshinesalesgroup.com | 3 |
| roles::427::0 | If someone is from San Antonio, TX, what season would their birthday fall in? | raw | false | Spring | 4 |
| roles::428::0 | During which season does the birthday of the person holding the position of Attending Physician occur? | raw | false | Winter | 3 |
| roles::429::0 | What are the main job responsibilities for someone with a high school education? | raw | false | Cultivate crops and raise livestock | 4 |
| roles::430::0 | What are the typical interests and hobbies of a person with a Bachelor's degree? | raw | false | A graceful sport that enhances coordination | 3 |
| roles::431::0 | For someone from Austin, TX, what would the sum of the last three digits of their phone number be? | raw | true | 13 | 4 |
| roles::432::0 | What is the sum of the last six digits of the contact number for the person who has the email address elijah.sawyer@urbanexcellencesg.com? | raw | true | 26 | 4 |
| roles::433::0 | What is the email address suffix for someone born on May 7th? | raw | true | @globallingServices.com | 4 |
| roles::434::0 | In which season does someone with a PhD celebrate their birthday? | raw | true | Autumn | 3 |
| roles::435::0 | What are the main responsibilities for someone whose birthday is on January 19th? | raw | true | Educate and guide students | 3 |
| roles::436::0 | How many letters are in the names of people who work in Austin, TX? | raw | false | 13 characters | 3 |
| roles::437::0 | What are Silas Bennett's main interests and hobbies? | raw | true | Nurture plants and get close to nature | 4 |
| roles::438::0 | What is the email address suffix used by members of the Austin Innovators Group? | raw | true | @austininnovatorsgroup.com | 3 |
| roles::439::0 | What is the email address suffix for people working in Boston, MA? | raw | true | @skywardhorizons.com | 3 |
| roles::440::0 | What would be the email address suffix for someone from Miami, FL? | raw | true | @silvercityhealthclinic.com | 4 |
| roles::441::0 | How many letters are in the name of the person who has the email address logan.carter@northeastfinancial.com? | raw | true | 11 characters | 3 |
| roles::442::0 | What is the sum of the last four digits of the contact number for the person who is 156 centimeters tall? | raw | false | 14 | 3 |
| roles::443::0 | What are the main interests and hobbies of someone who is a professional musician and composer? | raw | true | Aerobic exercise to improve cardiovascular health | 3 |
| roles::444::0 | What email address suffix would someone who is 158 cm tall use? | raw | true | @innovativesystemsengineering.com | 3 |
| roles::445::0 | What is the sum of the last two digits of the contact number for someone whose birthday is December 5th? | raw | true | 7 | 3 |
| roles::446::0 | What are the main responsibilities of someone who has swimming as a hobby? | raw | true | Perform various tasks on construction sites, including building, repairing, and maintaining structures | 3 |
| roles::447::0 | Which of the following descriptions accurately represents the workplace of someone who works in Washington, DC? | raw | true | The capital of the U.S., known for its national monuments and museums. | 3 |
| roles::448::0 | What are the key responsibilities of a 30-year-old person in their job? | raw | false | Provide quality service to passengers | 4 |
| roles::449::0 | Which of the following descriptions applies to someone who works in New York, NY? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 4 |
| roles::450::0 | Which of the following descriptions fits the work location of someone based in Washington, DC? | raw | true | The capital of the U.S., known for its national monuments and museums. | 3 |
| roles::451::0 | What is the sum of the last three digits of the contact number for the person who is 29 years old? | raw | true | 15 | 4 |
| roles::452::0 | What are the primary responsibilities of someone with a high school diploma in their job? | raw | true | Perform various tasks on construction sites, including building, repairing, and maintaining structures | 4 |
| roles::453::0 | What are Clara Whitman's main interests and hobbies? | raw | true | Experience fun in the virtual gaming world | 3 |
| roles::454::0 | What are the main responsibilities of a 28-year-old in their profession? | raw | false | Create innovative designs | 4 |
| roles::455::0 | In which season do Software Engineers usually celebrate their birthdays? | raw | false | Winter | 3 |
| roles::456::0 | When is the birthday of the person from Skyline Airways Inc., and what season does it fall in? | raw | false | Summer | 4 |
| roles::457::0 | What is the email address domain for a Police Officer? | raw | false | @guardiansafetyservices.com | 3 |
| roles::458::0 | Which of the following descriptions best describes the work location for someone located in Boston, MA? | raw | true | Known for its history, education, and sports teams. | 3 |
| roles::459::0 | What are the main responsibilities of an employee at Creative Canvas Studios? | raw | true | Create innovative designs | 4 |
| roles::460::0 | What describes the work location for someone who is based in Los Angeles, CA? | raw | true | Famous for Hollywood, beaches, and a vibrant arts scene. | 4 |
| roles::461::0 | What is the sum of the last two digits of the contact number for a person whose birthday is March 12th? | raw | true | 7 | 4 |
| roles::462::0 | During which season do Graphic Designers usually celebrate their birthdays? | raw | false | Spring | 3 |
| roles::463::0 | What would be a fitting description for someone who works in Miami, FL? | raw | true | Known for its beaches, nightlife, and multicultural atmosphere. | 3 |
| roles::464::0 | What would the email address suffix be for someone whose hobby is yoga? | raw | false | @neoninnovationslab.com | 4 |
| roles::465::0 | What is the email address suffix for a person who holds a PhD? | raw | false | @innovativeminds.edu | 4 |
| roles::466::0 | What are the primary interests and hobbies of a Creative Director? | raw | true | Appreciate theater and experience the variety of life | 3 |
| roles::467::0 | What are the main responsibilities of Lila Hawthorne in her profession? | raw | true | Teach and conduct research at a university level | 3 |
| roles::468::0 | Which of the following descriptions would apply to someone whose workplace is located in New York, NY? | raw | true | The largest city in the U.S., known for its iconic skyline and diverse culture. | 3 |
| roles::469::0 | What kind of work location would be suitable for someone based in Orlando, FL? | raw | true | Known for its theme parks, including Walt Disney World. | 4 |
| roles::470::0 | What is the sum of the last four digits of the contact number for the person from the Sunshine Sales Group? | raw | false | 15 | 4 |
| roles::471::0 | For someone whose workplace is in Washington, DC, which of the following descriptions applies to their job location? | raw | false | The capital of the U.S., known for its national monuments and museums. | 4 |
| roles::472::0 | What is the work location like for someone based in Portland, OR? | raw | false | Famous for its eco-friendliness and vibrant arts scene. | 4 |
| roles::473::0 | What are the main responsibilities of someone who practices yoga as a hobby? | raw | false | Educate and guide students | 3 |
| roles::474::0 | What are the primary duties of a 39-year-old in their profession? | raw | false | Create innovative designs | 3 |
| roles::475::0 | How many letters are there in the name of someone who is a professor? | raw | false | 11 characters | 3 |
| roles::476::0 | What is the sum of the last four digits of a golf enthusiast's contact number? | raw | false | 12 | 4 |
| roles::477::0 | In what season does a 29-year-old celebrate their birthday? | raw | false | Spring | 4 |
| roles::478::0 | What are the main responsibilities of someone born on March 9th? | raw | true | Uphold the law and provide legal services | 3 |
| roles::479::0 | What is the sum of the last four digits of the contact number for someone whose hobby is camping? | raw | false | 15 | 4 |
| roles::480::0 | How many letters are in the name of someone who is 157 cm tall? | raw | false | 13 characters | 4 |
| roles::481::0 | In which season does Gideon Cross have his birthday? | raw | false | Autumn | 4 |
| roles::482::0 | During which season does a doctor celebrate their birthday? | raw | true | Summer | 4 |
| roles::483::0 | What are the main responsibilities of a 25-year-old in their job? | raw | false | Maintain public safety and security | 3 |
| roles::484::0 | What are Ember Lawson's main interests and hobbies? | raw | true | Patiently wait and enjoy the pleasure of fishing | 4 |
| roles::485::0 | How many letters are there in the names of individuals who work in Austin, TX? | raw | false | 10 characters | 4 |
| roles::486::0 | What is the sum of the last three digits of the contact number for the person who is 24 years old? | raw | false | 19 | 4 |
| roles::487::0 | Which of the following descriptions would be a good fit for someone working in Miami, FL? | raw | false | Known for its beaches, nightlife, and multicultural atmosphere. | 4 |
| roles::488::0 | How many letters are in the name of a person who is 168 cm tall? | raw | true | 14 characters | 4 |
| roles::489::0 | What are the main responsibilities of the person with the contact number 85805107619? | raw | false | Promote products and achieve sales goals | 3 |
| roles::490::0 | What are the main interests and hobbies of a person who is 166 cm tall? | raw | true | Enhance fitness and maintain health | 4 |
| roles::491::0 | What are the main responsibilities of the person who has the email address elena.drake@lonestARretailgroup.com? | raw | true | Assist customers and promote products in retail environments | 4 |
| roles::492::0 | What season is the birthday of a person who is 158 cm tall? | raw | true | Summer | 3 |
| roles::493::0 | What would be the email address suffix for someone who is 151 cm tall? | raw | false | @orlandojusticepartners.com | 3 |
| roles::494::0 | What is the total of the last five digits of the contact number for a person whose birthday falls on February 15th? | raw | true | 25 | 4 |
| roles::495::0 | In which season does a 28-year-old celebrate their birthday? | raw | false | Autumn | 4 |
| roles::496::0 | How many letters are in the names of people who have a birthday on May 28th? | raw | true | 14 characters | 4 |
| roles::497::0 | If someone works in Chicago, IL, what season does their birthday fall in? | raw | true | Summer | 3 |
| roles::498::0 | What would be the email address suffix for someone named Dylan Carter? | raw | true | @harborviewmedicalcenter.org | 4 |
| roles::499::0 | What is the total of the last four digits of the contact number for a person whose birthday is on August 25th? | raw | true | 21 | 3 |