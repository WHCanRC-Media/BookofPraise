melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  bes'2 f2 d2 bes4 f'4 g4 a4 bes2 r2 \break

  % Line 2
  f2 bes4 a4 g2 f2 ees4 d4 c2 r2 \break

  % Line 3
  f2 g4 a4 bes4 a4 g2 f2 r2 \break

  % Line 4
  bes,2 d4 ees4 f4 f4 ees2 d2 c2 r2 \break

  % Line 5
  f2 g4 a4 bes4 c4 a2 g2 f2 r2 \break

  % Line 6
  f2 bes4 c4 ees4 d4 c2 bes1 \bar "|."
}
