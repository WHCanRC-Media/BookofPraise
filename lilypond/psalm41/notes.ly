melody = \relative c'' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 c4 c4 bes2 c2 d4 c4 bes2 a2 g4 \break

  % Line 2
  ees4 f4 g4 g4 fis4 g2 r2 \break

  % Line 3
  c2 g'2 aes2 g4 bes4 c4 d2 c2 bes4 c4 \break

  % Line 4
  g4 g4 f4 ees2 d2 c2 r2 \break

  % Line 5
  c2 ees4 d4 c2 g'2 g4 g4 aes2 f2 ees4 \break

  % Line 6
  g4 a4 bes4 c4 a4 g2 r2 \break

  % Line 7
  c2 g2 c2 bes4 g4 bes4 a4 g2 f2 ees4 \break

  % Line 8
  aes4 g4 f4 ees2 d2 c1 \bar "|."
}
