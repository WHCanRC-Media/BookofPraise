melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 c2 c4 c4 g'4 g4 bes4 bes4 f4 g4 a2 g2 r2 \break

  % Line 2
  c2 c4 c4 d2 c2 bes4 bes4 a4 a4 g2 r2 \break

  % Line 3
  c2 bes4 a4 g2 ees2 f4 g4 f4 ees4 d2 c2 r2 \break

  % Line 4
  c'2 c4 c4 d2 c2 bes4 bes4 a4 a4 g2 r2 \break

  % Line 5
  bes2 bes4 a4 g4 g4 aes4 g4 f4 ees4 f2 ees2 r2 \break

  % Line 6
  ees2 f4 aes4 g2 ees2 f4 ees4 d4 d4 c1 \bar "|."
}
