melody = \relative c' {
  \clef treble
  \key ees \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 c,4 d4 ees4 c4 c'4 aes4 g2 \break

  % Line 2
  ees2 aes4 g4 f4 ees4 f4 d4 c2 \break

  % Line 3
  g'2 bes4 aes4 d4 g,4 g4 fis4 g2 \break

  % Line 4
  bes2 f4 g4 aes4 g4 f4 f4 ees2 \break

  % Line 5
  f2 g4 ees4 bes'4 bes4 bes4 aes4 bes2 \break

  % Line 6
  g2 c4 d4 bes4 c4 c4 bes4 c2 \bar "|."
}
